import sys
import pandas as pd
import os
import re
from datetime import datetime

oracles = [
    # Add all oracles here
    'Acronis (Static ML)',
    'Ad-Aware',
    'AhnLab-V3',
    'ALYac',
    'Antiy-AVL',
    'Arcabit',
    'Avast',
    'Avira (no cloud)',
    'Baidu',
    'BitDefender',
    'BitDefenderTheta',
    'Bkav Pro',
    'ClamAV',
    'Comodo',
    'Cynet',
    'Cyren',
    'DrWeb',
    'EmsisoftAcronis (Static ML)',
    'Ad-Aware',
    'AhnLab-V3',
    'ALYac',
    'Antiy-AVL',
    'Arcabit',
    'Avast',
    'Avira (no cloud)',
    'Baidu',
    'BitDefender',
    'BitDefenderTheta',
    'Bkav Pro',
    'ClamAV',
    'Comodo',
    'Cynet',
    'Cyren',
    'DrWeb',
    'Emsisoft',
    'eScan',
    'ESET-NOD32',
    'F-Secure',
    'Fortinet',
    'GData',
    'Google',
    'Gridinsoft (no cloud)',
    'Ikarus',
    'Jiangmin',
    'K7AntiVirus',
    'K7GW',
    'Kaspersky',
    'Kingsoft',
    'Lionic',
    'Malwarebytes',
    'MAX',
    'McAfee',
    'McAfee-GW-Edition',
    'Microsoft',
    'NANO-Antivirus',
    'Panda',
    'QuickHeal',
    'Rising',
    'Sangfor Engine Zero',
    'Sophos',
    'SUPERAntiSpyware',
    'Symantec',
    'TACHYON',
    'Tencent',
    'Trellix (FireEye)',
    'TrendMicro',
    'TrendMicro-HouseCall',
    'VBA32',
    'VIPRE',
    'VirIT',
    'ViRobot',
    'Yandex',
    'Zillya',
    'ZoneAlarm by Check Point',
    'Zoner',
    'MaxSecure',
    'Alibaba',
    'Avast-Mobile',
    'BitDefenderFalx',
    'CrowdStrike Falcon',
    'Cybereason',
    'Cylance',
    'Elastic',
    'Palo Alto Networks',
    'SecureAge APEX',
    'SentinelOne (Static ML)',
    'Symantec Mobile Insight',
    'TEHTRIS',
    'Trapmine',
    'Trustlook',
    'Webroot',
    'eScan',
    'ESET-NOD32',
    'F-Secure',
    'Fortinet',
    'GData',
    'Google',
    'Gridinsoft (no cloud)',
    'Ikarus',
    'Jiangmin',
    'K7AntiVirus',
    'K7GW',
    'Kaspersky',
    'Kingsoft',
    'Lionic',
    'Malwarebytes',
    'MAX',
    'McAfee',
    'McAfee-GW-Edition',
    'Microsoft',
    'NANO-Antivirus',
    'Panda',
    'QuickHeal',
    'Rising',
    'Sangfor Engine Zero',
    'Sophos',
    'SUPERAntiSpyware',
    'Symantec',
    'TACHYON',
    'Tencent',
    'Trellix (FireEye)',
    'TrendMicro',
    'TrendMicro-HouseCall',
    'VBA32',
    'VIPRE',
    'VirIT',
    'ViRobot',
    'Yandex',
    'Zillya',
    'ZoneAlarm by Check Point',
    'Zoner',
    'MaxSecure',
    'Alibaba',
    'Avast-Mobile',
    'BitDefenderFalx',
    'CrowdStrike Falcon',
    'Cybereason',
    'Cylance',
    'Elastic',
    'Palo Alto Networks',
    'SecureAge APEX',
    'SentinelOne (Static ML)',
    'Symantec Mobile Insight',
    'TEHTRIS',
    'Trapmine',
    'Trustlook',
    'Webroot'
]
engines_re = r"(\d+)\n/ (\d+)"
# 2022-08-21 11:03:25 UTC
time_re = r"Size\n(\d\d\d\d)-(\d\d)-(\d\d) (\d\d):(\d\d):(\d\d) UTC"
# ai Score=74
MAX_ai_score = r"ai Score=(\d+)"

def parse_all_results_in_folder(folder):
    all = pd.DataFrame()
    for f in os.listdir(folder):
        if f.endswith(".logs.txt"):
            fr, _ = parse_result(f"{folder}/{f}")
            all = pd.concat([all, fr], axis = 0)
    return all

def parse_result(resultfile):
    lines = open(resultfile, "r").read()

    start = False
    oracle = None
    result = None
    results = []
    find = re.findall(engines_re, lines)
    detected = 0
    all_find = 0
    ai_score = 0
    if find:
        detected = int(find[0][0])
        if detected > 0 :
            print(resultfile)
        all_find = int(find[0][1])

    # find date
    date = re.findall(time_re, lines)
    if date:
        date = date[0]
        date = datetime(int(date[0]), int(date[1]), int(date[2]), int(date[3]), int(date[4]), int(date[5]))
    else:
        date = None

    aiscore_re = re.findall(MAX_ai_score, lines)
    if aiscore_re:
        print(aiscore_re)
        ai_score = int(aiscore_re[0])
    lines = open(resultfile, "r").readlines()
    for i, l in enumerate(lines):
        if not l:
            continue
        l = l.strip()

        if l in oracles:
            # check next line
            result = lines[i + 1]
            result = result.replace("\n", "")
            if result and result not in oracles:
                results.append((l,result))
            else:
                # then it is unknown
                results.append((l,"no_response"))

        
    #print(len(results), results)
    frame = pd.DataFrame(
        # slugify the value
        {
            'id': [os.path.basename(resultfile).replace(".logs.txt", "")],
            'non_benign': [detected],
            'engines': [all_find],
            'aiscore':  [ai_score],
            'date': [date],
            **{ k: [v.lower().replace(" ", "_")] for k, v in results }
        }
    )
    # return as pandas ?
    return frame, date


if __name__ == "__main__":
    all = parse_all_results_in_folder(sys.argv[1])
    #all = all[all['Kaspersky'] != 'undetected']
    all.to_csv("all.csv")
    #print(all)