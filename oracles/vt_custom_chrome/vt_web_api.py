from flask import Flask, request, redirect, make_response
from werkzeug.utils import secure_filename
import vt_web_gui
from flask_httpauth import HTTPBasicAuth
import parse_result
import os
import queue
import threading
import hashlib
import time
from io import StringIO
from werkzeug.security import generate_password_hash, check_password_hash

def server():
    app = Flask(__name__)

    worklist = queue.Queue()

    auth = HTTPBasicAuth()

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
        return f'Workcount {worklist.qsize()}'

    @app.route('/upload_file', methods=['GET', 'POST'])
    @auth.login_required
    def upload_file():
        if request.method == 'POST':
            # check if the post request has the file part
            print(request, request.data)
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
                file = open(os.path.join("upload", newname), 'wb')
                file.write(content)
                file.close()
                print(hash)
                # Adding to queue

                if os.path.exists(f"out/{hash}.wasm.logs.txt"):
                    print("Not queued")
                else:
                    print("Adding to queue")
                    worklist.put(os.path.join("upload", newname))
                return hash 
        return 'Enqueue a file'

    @app.route('/get_result/<hash>')
    @auth.login_required
    def get_analysis_result(hash):
        if os.path.exists(f"out/{hash}.wasm.logs.txt"):
            print("Loading result")
            f = parse_result.parse_result(f"out/{hash}.wasm.logs.txt")
            f.to_csv(f"upload/{hash}.csv")

            output = make_response(open(f"upload/{hash}.csv", "r").read())
            output.headers["Content-Disposition"] = f"attachment; filename=upload/{hash}.csv"
            output.headers["Content-type"] = "text/csv"
            return output

        # Return none if the hash was not yet added to the queue
        return 'INVALID'
    
    @app.route('/get_all_results')
    @auth.login_required
    def get_all_results():
        print("Loading result")
        f = parse_result.parse_all_results_in_folder(f"out")
        f.to_csv(f"upload/all.csv")

        output = make_response(open(f"upload/all.csv", "r").read())
        output.headers["Content-Disposition"] = f"attachment; filename=upload/all.csv"
        output.headers["Content-type"] = "text/csv"
        return output

    def check_files():

        WORKERS_NUMBER = int(os.environ.get("NO_WORKERS", "2"))

        prev = {}

        def process():

            while True:
                s = worklist.qsize()
                if s == 0:
                    print("Worklist empty, returning. Sleeping for a while")
                    time.sleep(5)

                filename = worklist.get()
                content = open(filename, "rb").read()
                hash = hashlib.sha256(content).hexdigest()
                if os.path.exists(f"out/{hash}.wasm.logs.txt"):
                    print(f"File {filename} already checked")
                    continue 

                worklist.task_done()
                print("Work count", s)
                times = 0        
                driver = vt_web_gui.setUp()

                done = False
                while times < 3:
                    try:
                        vt_web_gui.check_file(driver, filename, prev = prev)        
                        done = True
                        break
                    except Exception as e:
                        print(e)
                        times += 1
                        time.sleep(4*times)
                if not done:
                    # requeue the page
                    worklist.put(filename)

        workers = []
        for _ in range(WORKERS_NUMBER):
            th = threading.Thread(target=process)
            workers.append(th)
            th.start()
        
        return workers


    return app, check_files

# Run the workers in behind

if __name__ == '__main__':
    app, startfunc = server()
    threads = startfunc()
    app.run(host="0.0.0.0", port=4000, debug=False)

    for th in threads:
        th.join()