                    # requeue the page
                    with lock:
                        worklist.append((filename, outfolder, task))

        workers = []
        # Creating built in workers

        # The faster
        th = threading.Thread(target=process,args=[0], kwargs=dict(
            waiting_time_for_upload=0.01,
            waiting_time_for_analysis=0.1,
            waiting_time_for_hash=0.05,
            waiting_time_to_get_info=0.05,
            waiting_time_to_check_final=0.5,
            watiting_for_button_time=0.2,
            button_not_clicked_times=30
        ))
        th.start()
        workers.append(th)

        if not LOCAL:
            # The patientest
            th = threading.Thread(target=process, args=[1],kwargs=dict(
                waiting_time_for_upload=0.5,
                waiting_time_for_analysis=6,
                waiting_time_for_hash=0.9,
                waiting_time_to_get_info=0.2,
                waiting_time_to_check_final=4,
                watiting_for_button_time=2,
                button_not_clicked_times=1000
            ))
            workers.append(th)
            th.start()

            import random
            for i in range(WORKERS_NUMBER - 2):
                th = threading.Thread(target=process, args=[i + 2],kwargs=dict(

                    waiting_time_for_upload=random.randint(1, 10)/5.0,
                    waiting_time_for_analysis=random.randint(1, 10),
                    waiting_time_for_hash=random.randint(1, 10)/5.0,
                    waiting_time_to_get_info=random.randint(1, 10)/5.0,
                    waiting_time_to_check_final=random.randint(1, 26),
                    watiting_for_button_time=random.randint(1, 5)/2,
                    button_not_clicked_times=random.randint(1, 1000)
                ))
                workers.append(th)
                th.start()


        return workers


    def copy_folder():
        print("Copying folder locally, for better lookup")
        try:
            os.mkdir("./local_tmp")
        except Exception as e:
            print(e)

        # mcwrapper.loadfolder("./local_tmp/data", f"data")
        mcwrapper.loadfolder("./local_tmp/data", f"data")
        return "./local_tmp"

    return app, check_files, copy_folder

# Run the workers in behind


if __name__ == '__main__':



    app, startfunc, copy_folder = server()
    copy_folder()
    print("Initializing server")

    threads = startfunc()
    app.run(host="127.0.0.1", port=4000, debug=False)

    for th in threads:
        th.join()
