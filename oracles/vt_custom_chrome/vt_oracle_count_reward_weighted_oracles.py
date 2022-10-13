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

# Get this data from RQ1, what are the most resilient antiviruses ?
VENDORS_WEIGHT = {'Ad-Aware': 31, 'ALYac': 31, 'Arcabit': 28, 'BitDefender': 31, 'Emsisoft': 32, 'eScan': 31, 'Fortinet': 0, 'Google': 47, 'Ikarus': 60, 'Kaspersky': 133, 'MAX': 31, 'McAfee': 4, 'McAfee-GW-Edition': 4, 'Sangfor Engine Zero': 33, 'Symantec': 0, 'TrendMicro': 4, 'TrendMicro-HouseCall': 4, 'VIPRE': 31, 'Acronis (Static ML)': 0, 'AhnLab-V3': 0, 'Antiy-AVL': 0, 'Avast': 0, 'Avira (no cloud)': 0, 'Baidu': 0, 'BitDefenderTheta': 0, 'Bkav Pro': 0, 'ClamAV': 0, 'Comodo': 0, 'Cynet': 0, 'Cyren': 0, 'DrWeb': 0, 'ESET-NOD32': 0, 'F-Secure': 0, 'Gridinsoft (no cloud)': 0, 'Jiangmin': 2, 'K7AntiVirus': 0, 'K7GW': 0, 'Kingsoft': 0, 'Lionic': 0, 'Malwarebytes': 0, 'MaxSecure': 0, 'Microsoft': 0, 'NANO-Antivirus': 0, 'Panda': 0, 'QuickHeal': 0, 'Rising': 0, 'Sophos': 0, 'SUPERAntiSpyware': 0, 'TACHYON': 0, 'Tencent': 0, 'VBA32': 0, 'VirIT': 0, 'ViRobot': 0, 'Yandex': 0, 'Zillya': 4, 'ZoneAlarm by Check Point': 149, 'Zoner': 1}

def check_simple(oracleurl, checkoracle, user, pass_, session, input):
    global WHITELIST
    global VENDORS_WEIGHT
    
    # check count
    r = requests.get(
        f"{oracleurl}",
        auth = HTTPBasicAuth(user, pass_)
    )

    print(r.text)

    # submit the file
    # submit 2 times?
    for _ in range(2):
        r = requests.post(
            f"{oracleurl}/upload_file/{session}",
            files = { 'file': open(input, 'rb') },
            auth = HTTPBasicAuth(user, pass_)
        )

        hsh = r.text
    print(hsh)

    lapsed = 0
    waitfor = 5
    while lapsed <= 72000: # no more than 2 hours 

        r = requests.get(
            f"{oracleurl}/get_result/{session}/{hsh}",
            auth = HTTPBasicAuth(user, pass_)
        )
        if r.text != "INVALID":
            print(r.text)
            break

        lapsed += waitfor
        time.sleep(waitfor)

    print("Collecting result")


    DATA = StringIO(r.text)
    df = pd.read_csv(DATA)
    print(df)
    try:
        print("Non detected", df['non_benign'].values)

        val = 0
        for k in VENDORS_WEIGHT.keys():
            if k in df.columns:
                if df[k].values[0] in WHITELIST:
                    val += VENDORS_WEIGHT[k]
        if val == 0:
            sys.stderr.write(f"{val}")
            exit(0)
        else:
            sys.stderr.write(f"{val}")
            exit(1)
    except Exception as e:
        # This means an error on this proxy
        sys.stderr.write(f"{-1}")
        exit(0)


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
        hashes.append(dict(hsh=hsh, checked=False))

    # Then check hash by hash
    lapsed = 0
    waitfor = 5
    print("Collecting result")
    while True:
        complete = False
        for meta in hashes:
            try:
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
                if r.text == "INVALID":
                    continue

                DATA = StringIO(r.text)
                print(hsh)
                df = pd.read_csv(DATA)

                print(df)
                print("Non detected", df['non_benign'].values)

                val = df['non_benign'].values[0]


                if val == 0:
                    print("Not detected as mal")
                    exit(0)
                else:
                    sys.stderr.write(f"{val}")
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

    
