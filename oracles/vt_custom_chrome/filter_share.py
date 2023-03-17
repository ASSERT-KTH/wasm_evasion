import os
import sys

import requests
import hashlib
import json
from requests.auth import HTTPBasicAuth


if __name__ == "__main__":

    oracle_url = sys.argv[1]
    share_list_url = sys.argv[2]
    user = sys.argv[3]
    pass_ = sys.argv[4]

    # Download the list
    lst = requests.get(share_list_url)
    lst = lst.text
    lst = lst.split("\n")
    lst = [l for l in lst if l and not l.startswith("#")]

    print(f"Checking {len(lst)}")

    RESLT = []

    for l in lst:
        print(f"Checking {l}")
        r = requests.post(
            f"{oracle_url}/vt/detail/{l}",
                auth = HTTPBasicAuth(user, pass_)
            )

        isWasm = r.text
        if isWasm == 1:
            print(l, "Is Wasm")
            RESLT.append(l)

    open("result.json", "w").write(json.dumps(RESLT))


