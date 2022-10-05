import os

from selenium import webdriver
import os
from PIL import Image
import time
from selenium.webdriver.common.by import By
from selenium.webdriver.common.action_chains import ActionChains
from selenium.webdriver.common.keys import Keys
from concurrent.futures import ThreadPoolExecutor

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
    options.add_argument('--headless')
    options.add_argument("--enable-automation")
    options.add_argument("--dns-prefetch-disable")
    options.add_argument('window-size=1200x3000')
    #options.add_argument("--window-size=3200x20800")
    path = os.path.join(os.path.dirname(__file__), "chromedriver")
    
    driver = webdriver.Chrome(path, options=options)

    return driver