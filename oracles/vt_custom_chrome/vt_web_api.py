from flask import Flask, request, redirect, make_response
from werkzeug.utils import secure_filename
import vt_web_gui, vt_check_hash
from flask_httpauth import HTTPBasicAuth
import parse_result
import os
import threading
import hashlib
import time
import traceback
from werkzeug.security import generate_password_hash, check_password_hash
import vt_mc
import threading
import importlib
import subprocess
import os

COUNT = 0
DIRNAME = os.path.abspath(os.path.dirname(__file__))
LOCAL = False
TH = int(os.environ.get("TH", "58"))

def server():
    global COUNT
    global LOCAL
    global TH

    app = Flask(__name__)

    worklist = []
    lock = threading.Lock()

    auth = HTTPBasicAuth()
    if not LOCAL:
        mcwrapper = vt_mc.MCWrapper(
            os.environ.get("MC_ENDPOINT", "exp"),
            os.environ.get("MC_BUCKET", "my-bucket"),
            os.environ.get("MC_FILES", "vt_api_files")
            , local_cache="./local_tmp"
        )
    else:
        mcwrapper = vt_mc.LocalWrapper("out")


    users = {
        os.environ.get("WEB_USER", "admin"): generate_password_hash(os.environ.get("WEB_PASS", "admin"))
    }

    @auth.verify_password
    def verify_password(username, password):
        if username in users and \
                check_password_hash(users.get(username), password):
            return username

    @app.route('/')
    @auth.login_required
    def index():
        return f'Workcount {len(worklist)}'

    def upload_with_task(outfolder, request, task="SUBMIT"):
        global COUNT
        COUNT += 1
        print("Received count", COUNT)
        # check if the post request has the file part
        print(request)
        if 'file' not in request.files:
            return 'No file provided', 500
        file = request.files['file']
        # If the user does not select a file, the browser submits an
        # empty file without a filename.
        if file.filename == '':
            return 'No file provided', 500
        if file:
            content = file.read()
            hash = hashlib.sha256(content).hexdigest()

            newname = f"{hash}.wasm"

            mcwrapper.saveb(f"data/upload/{outfolder}/{newname}", content)
            print(hash)
            # Adding to queue
            try:
                if mcwrapper.exists(f"data/{outfolder}/{hash}.wasm.logs.txt"):
                    contentlog, hsh2 = mcwrapper.load(f"data/{outfolder}/{hash}.wasm.logs.txt")
                    tmp = open(f"/tmp/{hsh2}", 'wb')
                    try:
                        tmp.write(contentlog)
                    except Exception as e:
                        print(e)
                        tmp.write(contentlog.encode())

                    tmp.close()

                    f, _ = parse_result.parse_result(f"/tmp/{hsh2}")

                    if "Analysing (" not in contentlog or f['engines'].values[0] >= TH:
                        print("Not queued")
                    else:
                        print("Adding to queue")
                        # Make a tmp copy and send also to mcwrapper
                        tmpfile = f"/tmp/{hash}.wasm"
                        f = open(tmpfile, 'wb')
                        f.write(content)
                        f.close()
                        with lock:
                            worklist.append([f"/tmp/{hash}.wasm", f"{outfolder}", task ])

                else:
                    print("Adding to queue")
                    # Make a tmp copy and send also to mcwrapper
                    tmpfile = f"/tmp/{hash}.wasm"
                    f = open(tmpfile, 'wb')
                    f.write(content)
                    f.close()
                    with lock:
                        worklist.append([f"/tmp/{hash}.wasm", f"{outfolder}", task ])
            except Exception as e:
                print(e)
            return hash

    @app.route('/upload_file/<outfolder>', methods=['GET', 'POST'])
    @auth.login_required
    def upload_file(outfolder):
        global COUNT
        if request.method == 'POST':
            return upload_with_task(outfolder, request, "SUBMIT")

        return 'Enqueue a file'

    @app.route('/details/<outfolder>', methods=['GET', 'POST'])
    @auth.login_required
    def get_details(outfolder):
        global COUNT
        if request.method == 'POST':

            return upload_with_task(outfolder, request, "DETAILS")

        return 'Enqueue a file'

    @app.route('/vt/detail/<hash>', methods=['POST'])
    @auth.login_required
    def get_vt_on(hash):
        # Call the web gui
        if request.method == 'POST':
            driver = vt_web_gui.setUp()



            try:
                mod:vt_web_gui = importlib.reload(vt_web_gui)
                isWasm = mod.check_for_hash(driver, hash, out=f"data", wrapper=mcwrapper,                                    waiting_time_for_upload=0,
                                    waiting_time_for_analysis=0.01,
                                    waiting_time_for_hash=0.01,
                                    waiting_time_to_get_info=0.01,
                                    waiting_time_to_check_final=0.01,
                                    watiting_for_button_time=0.01,
                                    button_not_clicked_times=100
                                )
                return "1" if isWasm else "0"

            except Exception as e:
                print(e)
                return "0"

        return "0"



    @app.route('/get_result/<out>/<hash>')
    @auth.login_required
    def get_analysis_result(out, hash):


        if mcwrapper.exists(f"data/{out}/{hash}.wasm.logs.txt"):

            print("Loading result")
            # load first
            content, hsh = mcwrapper.load(f"data/{out}/{hash}.wasm.logs.txt")
            tmp = open(f"/tmp/{hsh}", 'wb')
            try:
                tmp.write(content)
            except Exception as e:
                print(e)
                tmp.write(content.encode())
            tmp.close()

            f, _ = parse_result.parse_result(f"/tmp/{hsh}")


            if b"Analysing (" not in content or f['engines'].values[0] >= TH:
                tmpcsv = f"/tmp/{hsh}.csv"
                f.to_csv(tmpcsv)

                output = make_response(open(tmpcsv, "r").read())
                # Save to mc
                mcwrapper.saveb(f"data/upload/{out}/{hash}.csv", open(tmpcsv, "rb").read())

                output.headers["Content-Disposition"] = f"attachment; filename=data/upload/{out}/{hash}.csv"
                output.headers["Content-type"] = "text/csv"
                return output
            else:
                print(content)
                print("Removing invalid result, asking to requeue")
                mcwrapper.remove(f"data/{out}/{hash}.wasm.logs.txt")
                return 'REQUEUE'

        # Return none if the hash was not yet added to the queue
        return 'INVALID'

    @app.route('/get_all_results/<out>')
    @auth.login_required
    def get_all_results(out):

        print("Loading result")
        # Copy all folder to tmp
        try:
            mcwrapper.loadfolder(f"/tmp/{out}", f"data/{out}")
            f = parse_result.parse_all_results_in_folder(f"/tmp/{out}")
            f.to_csv(f"/tmp/{out}/all.csv")

            output = make_response(open(f"/tmp/{out}/all.csv", "r").read())
            mcwrapper.save(f"data/upload/{out}/all.csv", open(f"/tmp/{out}/all.csv", "r").read())

            output.headers["Content-Disposition"] = f"attachment; filename=data/upload/{out}/all.csv"
            output.headers["Content-type"] = "text/csv"
            return output
        except Exception as e:
            # Empty response
            output = make_response(f"No folder found {e}")
            output.headers["Content-Disposition"] = f"attachment; filename=data/upload/none.txt"
            output.headers["Content-type"] = "text/csv"
            return output

    def check_files():

        WORKERS_NUMBER = int(os.environ.get("NO_WORKERS", "1"))

        prev = {}

        def process(
                waiting_time_for_upload=0.34,
                waiting_time_for_analysis=4,
                waiting_time_for_hash=0.6,
                waiting_time_to_get_info=0.3,
                waiting_time_to_check_final=2,
                watiting_for_button_time=2,
                button_not_clicked_times=500):
            # This should be a call to ray :)

            print(f"Crazy Ivan: waiting_time_for_upload: {waiting_time_for_upload} waiting_time_for_analysis {waiting_time_for_analysis} button_not_clicked_times {button_not_clicked_times}")

            while True:
                s = len(worklist)

                if s == 0:
                    print("Worklist empty. Sleeping for a while")
                    #worklist.task_done()
                    time.sleep(5)
                    continue

                with lock:
                    filename, outfolder, task = worklist.pop(0)

                content = open(filename, "rb").read()
                hash = hashlib.sha256(content).hexdigest()
                if mcwrapper.exists(f"data/{outfolder}/{hash}.wasm.logs.txt"):
                    content, hsh = mcwrapper.load(f"data/{outfolder}/{hash}.wasm.logs.txt")
                    tmp = open(f"/tmp/{hsh}", 'wb')
                    try:
                        tmp.write(content)
                    except Exception as e:
                        print(e)
                        tmp.write(content.encode())

                    tmp.close()

                    f, _ = parse_result.parse_result(f"/tmp/{hsh}")

                    if "Analysing (" not in content or f['engines'].values[0] >= TH:
                        print("Engines", f['engines'].values[0])
                        print(f"File {filename} already checked")
                        continue
                    else:
                        print("File existed before but was invalid")

                if mcwrapper.exists(f"data/{outfolder}/{hash}.details.txt"):
                    print(f"File {filename} already checked")
                    continue

                print("Work count", s)
                times = 0
                driver = vt_web_gui.setUp()

                done = False
                while times < 2:
                    try:
                        # reload module every time, this will help in Argo workflows
                        #print("Reloading module")
                        # subprocess.check_output(["/bin/bash", f"{DIRNAME}/download_module.sh"])

                        if task == 'SUBMIT':
                            mod:vt_web_gui = importlib.reload(vt_web_gui)
                            mod.check_file(driver, filename, prev = prev, out=f"data/{outfolder}", wrapper=mcwrapper,
                                    waiting_time_for_upload=waiting_time_for_upload,
                                    waiting_time_for_analysis=waiting_time_for_analysis,
                                    waiting_time_for_hash=waiting_time_for_hash,
                                    waiting_time_to_get_info=waiting_time_to_get_info,
                                    waiting_time_to_check_final=waiting_time_to_check_final,
                                    watiting_for_button_time=watiting_for_button_time,
                                    button_not_clicked_times=button_not_clicked_times)
                        elif task == 'DETAILS':
                            mod:vt_check_hash = importlib.reload(vt_check_hash)
                            mod.check_hash(driver, hash, wrapper=mcwrapper)

                        done = True
                        break
                    except Exception as e:
                        print(e)
                        print(traceback.format_exc())
                        if "net::ERR_PROXY_CONNECTION_FAILED" in traceback.format_exc():
                            # Restart proxy
                            print("Restarting")
                            open("name.socket", 'w').write("RESTART")
                            # Give time to restart
                            time.sleep(5)
                        times += 1
                        time.sleep(1)
                if not done:
                    # requeue the page
                    with lock:
                        worklist.append((filename, outfolder, task))

        workers = []
        # Creating built in workers

        # The faster
        th = threading.Thread(target=process, kwargs=dict(
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
            th = threading.Thread(target=process, kwargs=dict(
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
            for _ in range(WORKERS_NUMBER - 2):
                th = threading.Thread(target=process, kwargs=dict(

                    waiting_time_for_upload=random.randint(1, 10)/10.0,
                    waiting_time_for_analysis=random.randint(1, 10),
                    waiting_time_for_hash=random.randint(1, 10)/10.0,
                    waiting_time_to_get_info=random.randint(1, 10)/10.0,
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
