'''
    Takes a Wasm binary, and pass it to the oracle url as a file. Check the result and if its undetected or else in the specified antivirus, return 1
'''
import sys
import os
import requests
from requests.auth import HTTPBasicAuth
import time
import pandas as pd
from io import StringIO

WHITELIST = ['undetected', 'timeout', 'unable_to_process_file_type', 'no_response']


def check_simple(oracleurl, checkoracle, user, pass_, session, input):
    global WHITELIST

    # check count
    r = requests.get(
        f"{oracleurl}",
        auth = HTTPBasicAuth(user, pass_)
    )

    print(r.text)

    # submit the file
    r = requests.post(
        f"{oracleurl}/upload_file/{session}",
        files = { 'file': open(input, 'rb') },
        auth = HTTPBasicAuth(user, pass_)
    )

    hsh = r.text
    print(hsh)

    lapsed = 0
    waitfor = 5
    while lapsed <= 900: # no more than 15 mins

        r = requests.get(
            f"{oracleurl}/get_result/{session}/{hsh}",
            auth = HTTPBasicAuth(user, pass_)
        )
        if r.text == "REQUEUE":
            requests.post(f"{oracleurl}/upload_file/{session}",
                    files = { 'file': open(input, 'rb') },
                    auth = HTTPBasicAuth(user, pass_)
            )
        elif r.text != "INVALID":
            break

        lapsed += waitfor
        time.sleep(waitfor)

    print("Collecting result")


    DATA = StringIO(r.text)

    df = pd.read_csv(DATA)
    print(df)
    try:
        print("Non detected", df['non_benign'].values)

        val = df['non_benign'].values[0]
        engines = df['engines'].values[0]

        if val == 0 and engines >= 58:
            print("Not detected as mal")
            exit(1)
    except Exception as e:
        print(e)
        return check_simple(oracleurl, checkoracle, user, pass_, session, input)

def check_multiple(oracleurl, checkoracle, user, pass_, session,files):
    print(f"Processing {len(files)} files")
    global WHITELIST

    # check count
    r = requests.get(
        f"{oracleurl}",
        auth = HTTPBasicAuth(user, pass_)
    )

    print(r.text)


    # submit all the files first
    hashes = []
    for input in files:
        r = requests.post(
            f"{oracleurl}/upload_file/{session}",
            files = { 'file': open(input, 'rb') },
            auth = HTTPBasicAuth(user, pass_)
        )

        hsh = r.text
        print(hsh)
        hashes.append(dict(hsh=hsh, checked=False, f=input))

    # Then check hash by hash
    lapsed = 0
    waitfor = 1
    print("Collecting result")
    while True:
        complete = False
        for meta in hashes:
            if all(x['checked'] for x in hashes):
                complete = True
                break
            if meta['checked']:
                continue

            hsh = meta['hsh']
            r = requests.get(
                f"{oracleurl}/get_result/{session}/{hsh}",
                auth = HTTPBasicAuth(user, pass_)
            )

            if r.text == "REQUEUE":
                input = meta['f']
                requests.post(f"{oracleurl}/upload_file/{session}",
                        files = { 'file': open(input, 'rb') },
                        auth = HTTPBasicAuth(user, pass_)
                )
            elif r.text != "INVALID":
                continue

            DATA = StringIO(r.text)
            print(hsh)
            df = pd.read_csv(DATA)

            try:
                print(df)
                print("Non detected", df['non_benign'].values)

                val = df['non_benign'].values[0]
                engines = df['engines'].values[0]

                if val == 0:
                    print("Not detected as mal")
                    exit(1)
                # Remove hsh from hashes
                meta['checked'] = True
            except Exception as e:
                print(e)
                pass

        if complete:
            break

        time.sleep(waitfor)
        lapsed += waitfor

        if lapsed >= 900:
            break



if __name__ == '__main__':
    oracleurl = sys.argv[1]
    checkoracle = sys.argv[2]

    # auth to the service
    user = sys.argv[3]
    pass_ = sys.argv[4]
    session = sys.argv[5]
    input = sys.argv[6:] # all remainig files

    if len(input) == 1:
        check_simple(oracleurl, checkoracle, user, pass_, session, input[0])
    else:
        check_multiple(oracleurl, checkoracle, user, pass_, session, input)


