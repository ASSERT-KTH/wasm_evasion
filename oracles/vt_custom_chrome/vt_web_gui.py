from difflib import diff_bytes
from pickletools import optimize
import sys

from selenium import webdriver
import os
from PIL import Image
import time
from selenium.webdriver.common.by import By
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.keys import Keys
from concurrent.futures import ThreadPoolExecutor
import hashlib
import random
import queue
import threading
import traceback
import re
import filelock
import cv2
import pytesseract
from selenium.webdriver.common.action_chains import ActionChains
from datetime import datetime

#31
#/ 60
engines_re = r"(\d+)\n/ (\d+)"
TH = int(os.environ.get("TH", "58"))

def expand_element(driver, element, visited):
    subelements  = element.find_elements(By.XPATH, "./*")
    tag = element.get_attribute('tagName')
    class_ = element.get_attribute('class')

    text = element.text
    tags_to_skip= ["TEMPLATE" , "svg" , "g" ,"path" , "STYLE" , "img" , "video"  ]
    S =""


    if tag in tags_to_skip:
        return S
    S += f"tag: {tag}\n"
    S += f"class: {class_}\n"
    S += f"{text}\n"


    return S
    #shadowroot = expand_shadow_element(element)
    #if shadowroot:
     #   subelements  = shadowroot.find_elements(By.XPATH, "./*")

    #for obj in subelements:
    #    expand_element(obj, fd, visited)
    #    expand_element(obj, fd, visited)

def fullpage_screenshot(driver, name, file, from_="", callback=None):

        import uuid
        try:
            uniquefile = f"/tmp/{file}{uuid.uuid4()}.png"
            driver.get_screenshot_as_file(uniquefile)
            screenshot = Image.open(uniquefile)
            screenshot.save(uniquefile, optimize=True, quality=80)

            # send the screen shot to a callback
            if callback:
                callback(uniquefile)
            return screenshot
        except Exception as e:
            print(e)
            content = driver.find_element(By.TAG_NAME, 'body')

            print(expand_element(driver, content, {}))
            return None

def setUp():
    os.putenv('PREDEF_FILE', os.path.abspath("name.socket"))
    os.environ['PREDEF_FILE'] = os.path.abspath("name.socket")
    options = webdriver.ChromeOptions()
    PROXY = "socks5://127.0.0.1:9050" # IP:PORT or HOST:PORT
    options.add_argument('--proxy-server=%s' % PROXY)
    options.add_argument("disable-infobars"); # disabling infobars
    options.add_argument("--disable-extensions"); # disabling extensions
    options.add_argument("--disable-gpu"); # applicable to windows os only
    options.add_argument("--disable-dev-shm-usage"); # overcome limited resource problems
    options.add_argument("--no-sandbox"); #Bypass OS security model
    options.add_experimental_option("excludeSwitches",["ignore-certificate-errors"])
    # options.add_argument('--headless')
    options.add_argument("--enable-automation")
    options.add_argument("--dns-prefetch-disable")
    options.add_argument('window-size=1200x3000')
    #options.add_argument("--window-size=3200x20800")
    path = os.path.join(os.path.dirname(__file__), "chromedriver")


    driver = webdriver.Chrome(path, options=options)

    return driver

def check_files(files):

    WORKERS_NUMBER = int(os.environ.get("NO_WORKERS", "12"))

    worklist = queue.Queue()

    prev = {}

    def process():

        while True:
            s = worklist.qsize()
            if s == 0:
                print("Worklist empty, returning")
                break

            filename = worklist.get()
            worklist.task_done()
            print("Work count", s)
            times = 0
            driver = setUp()

            done = False
            while times < 2:
                try:
                    check_file(driver, filename, prev = prev)
                    print(f"{i}/{len(files)} {filename}")
                    done = True
                    break
                except Exception as e:
                    print(e)
                    print(traceback.format_exc())


                    if "net::ERR_PROXY_CONNECTION_FAILED" in traceback.format_exc():
                        print("Trying to access file")
                        with filelock.FileLock("name.socket.lock"):
                            # Restart proxy
                            print("Restarting")
                            f = open("name.socket", 'r+')
                            content = f.read()
                            if not "RESTART" in content:
                                f.seek(0)
                                f.write("RESTART")
                                f.close()
                            else:
                                print("Already restarting tor")

                            # Give time to restart
                            time.sleep(3 + 0.01*random.randint(1, 300))
                    times += 1
            if not done:
                # requeue the page
                worklist.put(filename)

    C = 0
    C2 = 0
    for i, filename in enumerate(files):
        # Check if exist
        content = open(filename, "rb").read()
        hash = hashlib.sha256(content).hexdigest()
        if os.path.exists(f"out/{hash}.wasm.logs.txt"):
            print(f"{C} File {filename} already checked")
            C += 1
        C2 += 1
        if C2 % 100 == 99:
            print(f"{C2}/{len(files)}")
        worklist.put(filename)

    print(f"Files count {worklist.qsize()}. Launching {WORKERS_NUMBER} workers")

    workers = []
    for _ in range(WORKERS_NUMBER):
        th = threading.Thread(target=process)
        workers.append(th)
        th.start()

    for th in workers:
        th.join()

    #for j in jobs:
    #    j.result()




def expand_shadow_element(driver, element):
    try:
        shadow_root = driver.execute_script('return arguments[0].shadowRoot', element)
        return shadow_root
    except Exception as e:
        print(e)
        return None

def break_if_captcha(driver, name):
    #image = fullpage_screenshot(driver, name, f"snapshots/{name}.analysis.png")

    content = driver.find_element(By.TAG_NAME, 'body')
    content_text = expand_element(driver, content, {})


    if "captcha" in driver.current_url or "RayID" in content_text or "Forbidden" in content_text:

        print("Trying to access file")
        with filelock.FileLock("name.socket.lock"):
            print("Restarting")
            f= open("name.socket", 'r+')
            content = f.read()
            if not "RESTART" in content:
                f.seek(0)
                f.write("RESTART")
                f.close()
            else:
                print("Already restarting tor")
            raise Exception("Blocked. Restarting tor ?")

def get_confirm_btn_position(driver, name, wrapper):
    image = fullpage_screenshot(driver,name, f"{name}.png")
    # Detect where the button is
    image = cv2.imread(f"{name}.png")

    # wrapper.savefile(f"screenshots/{name}.upload.png", f"{name}.png")
    gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
    # Performing OTSU threshold
    ret, thresh1 = cv2.threshold(gray, 0, 255, cv2.THRESH_OTSU | cv2.THRESH_BINARY_INV)

    # Specify structure shape and kernel size.
    # Kernel size increases or decreases the area
    # of the rectangle to be detected.
    # A smaller value like (10, 10) will detect
    # each word instead of a sentence.
    rect_kernel = cv2.getStructuringElement(cv2.MORPH_RECT, (21, 21))
    rect_kernel2 = cv2.getStructuringElement(cv2.MORPH_RECT, (2, 2))


    # Erode first to remove "Choose file border"
    dilation = cv2.erode(thresh1, rect_kernel2, iterations = 1)
    cv2.imwrite(f"{name}.gray1.png", dilation)
    # Applying dilation on the threshold image
    dilation = cv2.dilate(dilation, rect_kernel, iterations = 1)


    cv2.imwrite(f"{name}.gray2.png", dilation)
    # Finding contours
    contours, hierarchy = cv2.findContours(dilation, cv2.RETR_EXTERNAL,
                                                    cv2.CHAIN_APPROX_NONE)
    im2 = image.copy()
    for cnt in contours:
        x, y, w, h = cv2.boundingRect(cnt)

        # Drawing a rectangle on copied image
        rect = cv2.rectangle(im2, (x, y), (x + w, y + h), (0, 255, 0), 2)

        # Cropping the text block for giving input to OCR
        cropped = im2[y:y + h, x:x + w]
        text = pytesseract.image_to_string(cropped)
        if text.strip() in ["Confirm upload", "Confirm", "Confir", "Confi", "Conf"]:
            print(text)
            cv2.imwrite(f"{name}.rect.png", im2)
            # wrapper.savefile(f"screenshots/{name}.rect.png", f"{name}.rect.png")


            return (x, y), (h, w)
    return None, None

def get_submit_btn_position(driver):
    pass



def check_file(driver, filename, prev = {}, out="out", wrapper = None, callback = None,
    waiting_time_for_upload=0.34,
    waiting_time_for_analysis=4,
    waiting_time_for_hash=0.6,
    waiting_time_to_get_info=0.3,
    waiting_time_to_check_final=2,
    watiting_for_button_time=2,
    button_not_clicked_times=500
):
    # create a debug monitor to take screenshot
    name = os.path.basename(filename)


    url = "https://www.virustotal.com/gui/home/upload"
    driver.delete_all_cookies()
    #driver.maximize_window()
    #driver.set_window_size(300, 1800)
    actions = ActionChains(driver)



    print(f"Taking {name}")
    driver.get(url)



    print(f"Taking {name}")
    driver.get(url)

    # To avoid bot
    # . time.sleep(random.randint(1,3))
    print("Waiting for upload btn")
    break_if_captcha(driver, name)
    times = 0
    while True:
        fullpage_screenshot(driver, name, f"meh.init.png",from_="Waiting from file hash", callback=callback)
        if "file" in driver.current_url:
            break
        time.sleep(waiting_time_for_upload)
        times += 1

        if times >= 20:
            print("Restarting")
            raise Exception("Too many times")
        # Detect where the button is
        # #infoIcon
        break_if_captcha(driver, name)
        #fullpage_screenshot(driver, name, f"{name}.init.png",from_="Waiting from file hash")
        #wrapper.savefile(f"screenshots/{name}.init.png", f"{name}.init.png")

        fullpage_screenshot(driver, name, f"snapshots/{name}.upload.png",from_="Waiting from upload btn", callback=callback)
        try:
            inpt = driver.execute_script("return document.querySelector('vt-ui-shell').querySelector('#view-container home-view').shadowRoot.querySelector('vt-ui-main-upload-form').shadowRoot.querySelector('#fileSelector')")
            break
        except:
            #print(traceback.format_exc())

            pass
    driver.execute_script("arguments[0].style.display = 'block';", inpt)
    #print(inpt)
    inpt.send_keys(os.path.abspath(filename))

    time.sleep(watiting_for_button_time)
    # Now confirm the upload if needed
    times = 0
    print("Checking if confirm button")
    while True:
        if "file" in driver.current_url:
            break
        time.sleep(0.05)
        break_if_captcha(driver, name)

        try:
            try:
                btn = driver.execute_script("return document.querySelector('vt-ui-shell').querySelector('#view-container home-view').shadowRoot.querySelector('vt-ui-main-upload-form').shadowRoot.querySelector('#confirmUpload')")
            except Exception as e:
                print(e)
                btn = None
            if btn:
                print("Confirming upload")
                driver.execute_script("arguments[0].click();", btn)
                break
            else:
                # Try with the screenshot...this takes time, so we try just is the button does not exist
                #print("Doing image based detection", name)
                buttonpos, size = (1123, 1055),(39.0*2, 172.5*2) # get_confirm_btn_position(driver, name, wrapper)
                print("Position", buttonpos, name)
                if buttonpos:
                    x, y = buttonpos
                    x = x/2
                    y = y/2

                    h, w = size

                    h = h/2 - 5
                    w = w/2 - 5


                    print("Button found", x, y, h, w)
                    #driver.set_window_size(2400, 1800)


                    #actions.move_by_offset(x, y).click().perform()
                    # create a marker in the page to show where the mouse is
                    driver.execute_script(f" dot = document.createElement('div'); dot.id='marker', dot.style.position = 'absolute'; dot.style.top = '0px'; dot.style.left = '0px'; dot.style.width = '{w}px'; dot.style.height = '{h}px'; dot.style.backgroundColor = 'red'; dot.style.opacity=0.3; document.body.appendChild(dot);")
                    driver.execute_script(f" dot = document.createElement('div'); dot.id='marker2', dot.style.position = 'absolute'; dot.style.top = '{y - 1}px'; dot.style.left = '{x - 1}px'; dot.style.width = '5px'; dot.style.height = '5px'; dot.style.backgroundColor = 'blue'; dot.style.opacity=0.3; document.body.appendChild(dot);")

                    fullpage_screenshot(driver, name, f"{name}.click.png",from_="Waiting from file hash", callback=callback)
                    #wrapper.savefile(f"screenshots/{name}.click.png", f"{name}.click.png")
                    f = open(f"/tmp/url{name}",  "w")
                    f.write(f"{driver.current_url}")
                    f.close()
                    wrapper.savefile(f"screenshots/{name}.url.txt", f"/tmp/url{name}")



                    marker = driver.find_element(By.ID, "marker")
                    # print(marker, name)
                    actions.move_to_element_with_offset(marker, x, y).click().perform()

                    # Remove the element
                    #driver.execute_script("document.getElementById('marker').remove();")
                    #time.sleep(1)
                    #time.sleep(1)

                    #break
            times += 1
            if times > button_not_clicked_times:
                print("Button not clicked ?")
                raise Exception("Too many times")
                break
        except Exception as e:
            print(e)
            print(traceback.format_exc())

            pass

    content_text = ""

    time.sleep(waiting_time_for_analysis)
    print("Wait for the analysis", name)
    times = 0
    while "/file-analysis/" in driver.current_url:
        times += 1

        if times >= 300:
            raise Exception("Too many times")

        break_if_captcha(driver, name)
        time.sleep(1)
        print("Yet analysing...", driver.current_url)
        content = driver.find_element(By.TAG_NAME, 'body')
        content_text = expand_element(driver, content, {})
        # image = fullpage_screenshot(driver, name, f"wrong/{name}.analysis.png",from_="Waiting from analysis")


    times = 0
    print("Getting info from file hash address",driver.current_url)
    while "/file/" not in driver.current_url:
        break_if_captcha(driver, name)
        print(driver.current_url, times)
        # Take an screenshot and save it
        time.sleep(waiting_time_for_hash)
        times += 1
        if times >= 60: #360s
            raise Exception("Wait too much")

    #time.sleep(2)
    # / 54
    if "file" in driver.current_url:
        times = 0
        while True:
            time.sleep(waiting_time_to_get_info)
            break_if_captcha(driver, name)
            # Done
            content = driver.find_element(By.TAG_NAME, 'body')
            content_text = expand_element(driver, content, {})
            times += 1
            if times >= 80: # 600s 10mins
                image = fullpage_screenshot(driver, name, f"{name}.recogn.png",from_="Timeout", callback=callback)
                raise Exception("Waiting too much")

            matches = re.findall(engines_re, content_text)
            if matches:
                print("Analysis", name, matches, times, "Analysing (" in content_text, TH)
                positives = matches[0][0]
                positives = int(positives)
                all = matches[0][1]
                all = int(all)

                if (all >= TH or "Security Vendors' Analysis" in content_text) and "Analysing (" not in content_text:
                    print("Returning")
                else:
                    continue

                if not wrapper:
                    fd = open(f"{out}/{name}.logs.txt", "w")
                    fd.write(content_text)
                    fd.close()
                else:
                    # set the date as the first line
                    now = datetime.now()
                    content_text = f"DATE: {now}\n{content_text}"
                    wrapper.save(f"{out}/{name}.logs.txt", content_text)

                image = fullpage_screenshot(driver, name, f"{name}.recogn.png",from_="Waiting from file hash", callback=callback)

                if wrapper:
                    wrapper.savefile(f"{out}/{name}.recogn.png", f"{name}.recogn.png")

                print(f"Done {name}")
                return

    print("Wrong result")
    #time.sleep(3)
    image = fullpage_screenshot(driver, name, f"wrong/{name}wrong.png",from_="wrong result", callback=callback)
    raise Exception("Wrong result")

if __name__ == "__main__":
    files = os.listdir(sys.argv[1])
    files = [f"{sys.argv[1]}/{f}" for f in files if f.endswith(".wasm")]
    check_files(files)

