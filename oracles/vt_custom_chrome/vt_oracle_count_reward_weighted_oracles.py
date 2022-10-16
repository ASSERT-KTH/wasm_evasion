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
VENDORS_WEIGHT = {'ZoneAlarm by Check Point': 13, 'Kaspersky': 13, 'Ikarus': 8, 'Google': 8, 'Emsisoft': 6, 'eScan': 6, 'VIPRE': 6, 'Ad-Aware': 6, 'ALYac': 6, 'MAX': 6, 'BitDefender': 6, 'Arcabit': 6, 'Sangfor Engine Zero': 5, 'McAfee': 3, 'McAfee-GW-Edition': 3, 'Symantec': 3, 'TrendMicro-HouseCall': 2, 'TrendMicro': 2, 'Zillya': 1, 'Sophos': 1, 'Fortinet': 1, 'Kingsoft': 1, 'Zoner': 1, 'ESET-NOD32': 1, 'VirIT': 1, 'Lionic': 1, 'Microsoft': 1, 'Jiangmin': 1, 'MaxSecure': 1, 'ClamAV': 1, 'VBA32': 1, 'ViRobot': 1, 'Antiy-AVL': 1, 'DrWeb': 1, 'Rising': 1, 'K7AntiVirus': 1, 'F-Secure': 1, 'Gridinsoft (no cloud)': 1, 'Yandex': 1, 'Comodo': 1, 'TACHYON': 1, 'Cynet': 1, 'K7GW': 1, 'Malwarebytes': 1, 'Avira (no cloud)': 1, 'Tencent': 1, 'NANO-Antivirus': 1, 'QuickHeal': 1, 'AhnLab-V3': 1, 'Bkav Pro': 1, 'SUPERAntiSpyware': 1, 'Avast': 1, 'Baidu': 1, 'Panda': 1, 'Cyren': 1, 'Acronis (Static ML)': 1, 'BitDefenderTheta': 1}

MAX = 0

for k in VENDORS_WEIGHT:
    MAX += VENDORS_WEIGHT[k]


def check_simple(oracleurl, checkoracle, user, pass_, session, input):
    global WHITELIST
    global VENDORS_WEIGHT
    global MAX
    
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
        if val == MAX:
            sys.stderr.write(f"{val}")
            exit(0)
        else:
            sys.stderr.write(f"{val}")
            exit(1)
    except Exception as e:
        # This means an error on this proxy
        # requeue
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

    
