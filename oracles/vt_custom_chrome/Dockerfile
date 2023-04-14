From ubuntu:20.04

RUN apt-get update
RUN apt-get install -y software-properties-common
RUN apt-get -y install tesseract-ocr
RUN apt-get -y  install libtesseract-dev
RUN add-apt-repository universe
RUN apt-get update
RUN apt-get install -y python3 tor zip unzip wget python3-pip

RUN wget -O google-chrome-stable_current_amd64.deb https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
RUN dpkg -i google-chrome-stable_current_amd64.deb || true
RUN apt-get install -y -f

RUN google-chrome --version

RUN mkdir /data
RUN mkdir /vt_oracle
WORKDIR /vt_oracle

COPY vt_web_gui.py .
COPY vt_mc.py .
COPY vt_web_api.py .
COPY vt_check_hash.py .
COPY vt_oracle_count_reward_weighted_oracles.py .
COPY setup.py .
COPY parse_result.py .
COPY vt_oracle_count_reward.py .
COPY tor.sh .
COPY requirements.txt .
COPY test.wasm /data/test.wasm
COPY entrypoint.sh .
COPY test.py .
COPY test.wasm test.wasm
COPY download_module.sh .

RUN mkdir out
RUN mkdir upload
RUN mkdir wrong
RUN mkdir snapshots


RUN pip3 install -r requirements.txt
RUN wget -O chromedriver.zip https://chromedriver.storage.googleapis.com/112.0.5615.49/chromedriver_linux64.zip
RUN unzip chromedriver.zip
RUN rm chromedriver.zip
RUN ln chromedriver /bin/chromedriver
# RUN mv chromedriver /bin/chromedriver
RUN touch name.socket


RUN mkdir -p $HOME/minio-binaries
RUN wget -O $HOME/minio-binaries/mc https://dl.min.io/client/mc/release/linux-amd64/mc
RUN ls $HOME/minio-binaries/mc 
RUN chmod +x $HOME/minio-binaries/mc
# RUN export PATH=$PATH:$HOME/minio-binaries/
RUN ln $HOME/minio-binaries/mc /bin/mc
RUN mc --version


RUN ls .

ENV NO_WORKERS "8"
ENV WEB_USER "admin"
ENV WEB_PASS  "admin"
ENV MC_ENDPOINT "exp"
ENV MC_BUCKET "my-bucket"

ENV MINIO_HOST "http://minio-service.minio:3434"
ENV MINIO_PASS "minio123"
ENV MINIO_USER = "minio"
ENV MC_FILES = "vt-api-files"
# RUN test
RUN python3 test.py

ENTRYPOINT ["/bin/bash", "entrypoint.sh"]
# Download the driver
