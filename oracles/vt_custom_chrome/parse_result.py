import sys
import pandas as pd
import os

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

def parse_all_results_in_folder(folder):
    all = pd.DataFrame()
    for f in os.listdir(folder):
        if f.endswith(".logs.txt"):
            fr = parse_result(f"{folder}/{f}")
            all = pd.concat([all, fr], axis = 0)
    return all

def parse_result(resultfile):
    print(resultfile)
    lines = open(resultfile, "r").readlines()

    start = False
    oracle = None
    result = None
    results = []

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
            **{ k: [v.lower().replace(" ", "_")] for k, v in results }
        }
    )
    # return as pandas ?
    return frame


if __name__ == "__main__":
    all = parse_all_results_in_folder(sys.argv[1])
    #all = all[all['Kaspersky'] != 'undetected']
    all.to_csv("all.csv")
    #print(all)