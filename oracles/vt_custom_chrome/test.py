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

#31
#/ 60
engines_re = r"(\d+)\n/ (\d+)"

def fullpage_screenshot(driver, name, file, from_=""):

        driver.get_screenshot_as_file(file)
        screenshot = Image.open(file)
        screenshot.save(file, optimize=True, quality=80)

        return screenshot


def get_confirm_btn_position(driver, name):
    image = fullpage_screenshot(driver,name, f"{name}.png")
    # Detect where the button is
    image = cv2.imread(f"{name}.png")
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

            return (x, y), (h, w)
    return None, None


if __name__ == "__main__":
    options = webdriver.ChromeOptions()
    options.add_argument("disable-infobars"); # disabling infobars
    options.add_argument("--disable-extensions"); # disabling extensions
    options.add_argument("--disable-gpu"); # applicable to windows os only
    options.add_argument("--disable-dev-shm-usage"); # overcome limited resource problems
    options.add_argument("--no-sandbox"); #Bypass OS security model
    options.add_argument("--enable-automation")
    options.add_argument("--dns-prefetch-disable")
    options.add_experimental_option("excludeSwitches",["ignore-certificate-errors"])
    options.add_argument('--headless')
    options.add_argument('window-size=1200x3000')
    #options.add_argument("--window-size=3200x20800")
    path = os.path.join(os.path.dirname(__file__), "chromedriver")
    
    driver = webdriver.Chrome(path, options=options)

    driver.get("https://google.com")
    get_confirm_btn_position(driver, "google")

    # Check tesseract

