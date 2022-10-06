from curses import wrapper
from setup import setUp
import os
from vt_web_gui import expand_element, fullpage_screenshot,break_if_captcha
import time
from selenium.webdriver.common.by import By
import filelock

def check_hash(driver, hash, wrapper=None):
   
    url = f"https://www.virustotal.com/gui/file/{hash}/details"
    driver.delete_all_cookies()
    
    print(f"Visiting {url}")
    driver.get(url)

    time.sleep(1)
    break_if_captcha(driver, hash) 

    # Get the text output

    content = driver.find_element(By.TAG_NAME, 'body')
    content_text = expand_element(driver, content, {})

    if "Forbidden" in content_text:
        # This is for debugging reasons, comment out in deplot since this generation is expensive
        fullpage_screenshot(driver, "details", f"out/{hash}.details.png")
        print( f"out/{hash}.details.png")
        raise Exception("Forbidden") 

    if "History" not in content_text:
        print(content_text)
        raise Exception("No history found")

    f = open(f"out/{hash}.details.txt", "w")
    f.write(content_text)
    f.close()

    if wrapper:
        wrapper.save(f"out/{hash}.details.txt", content_text)
    