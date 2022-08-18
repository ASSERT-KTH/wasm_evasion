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

if __name__ == '__main__':
    WHITELIST = ['undetected', 'timeout', 'unable_to_process_file_type', 'no_response']
    oracleurl = sys.argv[1]
    checkoracle = sys.argv[2]
    
    # auth to the service
    user = sys.argv[3]
    pass_ = sys.argv[4]

    input = sys.argv[5]

    # check count
    r = requests.get(
        f"{oracleurl}",
        auth = HTTPBasicAuth(user, pass_)
    )

    print(r.text)

    # submit the file
    r = requests.post(
        f"{oracleurl}/upload_file",
        files = { 'file': open(input, 'rb') },
        auth = HTTPBasicAuth(user, pass_)
    )

    hsh = r.text
    print(hsh)

    lapsed = 0
    waitfor = 5
    while lapsed <= 900: # no more than 15 mins

        r = requests.get(
            f"{oracleurl}/get_result/{hsh}",
            auth = HTTPBasicAuth(user, pass_)
        )
        if r.text != "INVALID":
            break

        lapsed += waitfor
        time.sleep(waitfor)

    print("Collecting result")


    DATA = StringIO(r.text)

    df = pd.read_csv(DATA)
    print(df)
    print(df[checkoracle].values)

    val = df[checkoracle].values[0]

    if val in WHITELIST:
        print("Not detected as mal")
        exit(1)
    
