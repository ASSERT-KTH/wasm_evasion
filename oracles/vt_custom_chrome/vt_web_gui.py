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

