from difflib import diff_bytes
import sys

from selenium import webdriver
import unittest
import os
from PIL import Image
import time
import numpy as np
from subprocess import check_output
import shutil
from selenium.webdriver.common.by import By
from selenium.webdriver.common.action_chains import ActionChains
import cv2
import pytesseract
import socket,os
from selenium.webdriver.common.keys import Keys
from concurrent.futures import ThreadPoolExecutor
import hashlib


def fullpage_screenshot(driver, file):

        print("Starting chrome full page screenshot workaround ...")

        total_width = driver.execute_script("return document.body.offsetWidth")
        total_height = driver.execute_script("return document.body.parentNode.scrollHeight")
        viewport_width = driver.execute_script("return document.body.clientWidth")
        viewport_height = driver.execute_script("return window.innerHeight")
        print("Total: ({0}, {1}), Viewport: ({2},{3})".format(total_width, total_height,viewport_width,viewport_height))
        rectangles = []

        i = 0
        while i < total_height:
            ii = 0
            top_height = i + viewport_height

            if top_height > total_height:
                top_height = total_height

            while ii < total_width:
                top_width = ii + viewport_width

                if top_width > total_width:
                    top_width = total_width

                print("Appending rectangle ({0},{1},{2},{3})".format(ii, i, top_width, top_height))
                rectangles.append((ii, i, top_width,top_height))

                ii = ii + viewport_width

            i = i + viewport_height

        stitched_image = Image.new('RGB', (total_width, total_height))
        previous = None
        part = 0

        for rectangle in rectangles:
            if not previous is None:
                driver.execute_script("window.scrollTo({0}, {1})".format(rectangle[0], rectangle[1]))
                print("Scrolled To ({0},{1})".format(rectangle[0], rectangle[1]))
                time.sleep(0.2)

            file_name = "part_{0}.png".format(part)
            print("Capturing {0} ...".format(file_name))

            driver.get_screenshot_as_file(file_name)
            screenshot = Image.open(file_name)

            if rectangle[1] + viewport_height > total_height:
                offset = (rectangle[0], total_height - viewport_height)
            else:
                offset = (rectangle[0], rectangle[1])

            print("Adding to stitched image with offset ({0}, {1})".format(offset[0],offset[1]))
            stitched_image.paste(screenshot, offset)

            del screenshot
            os.remove(file_name)
            part = part + 1
            previous = rectangle

        stitched_image.save(file, optimize=True, quality=95)
        print("Finishing chrome full page screenshot workaround...")
        return stitched_image

def setUp():
    os.putenv('PREDEF_FILE', os.path.abspath("name.socket"))
    os.environ['PREDEF_FILE'] = os.path.abspath("name.socket")
    options = webdriver.ChromeOptions()
    options.binary_location = '/Users/javierca/Documents/Develop/chromium/src/out/Default/Chromium.app/Contents/MacOS/Chromium'
    options.add_experimental_option("excludeSwitches",["ignore-certificate-errors"])
    #options.add_argument('--headless')
    options.add_argument('window-size=1200x1000')
    path = os.path.join(os.path.dirname(__file__), "chromedriver")
    
    driver = webdriver.Chrome(path, options=options)

    return driver

def check_files(files):

    prev = {}
    times = 0
    pool = ThreadPoolExecutor(1)

    jobs = []

    def process(i, filename):
        # Check if exist
        content = open(filename, "rb").read()

        hash = hashlib.sha256(content).hexdigest()

        if os.path.exists(f"out/{hash}.wasm.logs.txt"):
            print(f"File {i}/{len(files)} {filename} already checked")
            return 

        times = 0        
        driver = setUp()

        while times < 3:
            try:
                check_file(driver, filename, prev = prev)        
                print(f"{i}/{len(files)} {filename}")
                break
            except Exception as e:
                print(e)
                times += 1
                time.sleep(4*times)

    for i, filename in enumerate(files):
        jobs.append(pool.submit(process, i, filename))

    for j in jobs:
        j.result()
        

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

def expand_shadow_element(driver, element):
    try:
        shadow_root = driver.execute_script('return arguments[0].shadowRoot', element)
        return shadow_root
    except Exception as e:
        print(e)
        return None

def check_file(driver, filename, prev = {}):
    name = os.path.basename(filename)
    try:
        os.remove("name.socket")
    except OSError:
        pass

    fifo = open("name.socket", "w")
    fifo.write(os.path.abspath(filename))
    fifo.write("\n")
    fifo.close()
    url = "https://www.virustotal.com/gui/home/upload"
    driver.delete_all_cookies()
    driver.get(url)
    
    while True:
        # Detect where the button is
        # #infoIcon
        try:
            inpt = driver.execute_script("return document.querySelector('vt-ui-shell').querySelector('#view-container home-view').shadowRoot.querySelector('vt-ui-main-upload-form').shadowRoot.querySelector('#fileSelector')")
            break
        except: 
            pass
    driver.execute_script("arguments[0].style.display = 'block';", inpt)
    print(inpt)
    inpt.send_keys(os.path.abspath(filename))

    content_text = ""

    while "Undetected" not in content_text:
        print("Getting")
        content = driver.find_element(By.TAG_NAME, 'body')
        content_text = expand_element(driver, content, {})

    fd = open(f"out/{name}.logs.txt", "w")
    fd.write(content_text)
    fd.close()
    image = fullpage_screenshot(driver, f"out/{name}recogn.png")


if __name__ == "__main__":
    files = os.listdir(sys.argv[1])
    files = [f"{sys.argv[1]}/{f}" for f in files if f.endswith(".wasm")]
    check_files(files)