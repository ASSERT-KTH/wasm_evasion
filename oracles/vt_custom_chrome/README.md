## VT oracle

This is a wrapper to access the VirusTotal report on a submited file. Here, we simulate a user submitting a file by hand (puppeteer) and we then parse the resulting HTML to collect the labels from all reported vendors.

To deploy the oracle as an API:
- Run the `tor.sh` script.
- Run `python3 vt_web_api.py`. This will launch a flask application, set the env var `WEB_USER` and `WEB_PASS` to set the user and password in the API.

The API has two main endpoints: `upload_file` and `get_result`. With the first one you can upload a binary to check and, since the process can take several minutes in VirusTotal, by polling the `get_result` endpoint with the hash returned by `upload_file`, you can get the VT report on the file.


The scripts `vt_oracle_count.py` and `vt_oracle_count_reward.py` use the previous described endpoints. You can execute those scripts and pass a file path as the first argument. The scripts then upload the binary and constantly poll the VT oracle until they can return in the exit code, the aggregation of the VirusTotal result.
