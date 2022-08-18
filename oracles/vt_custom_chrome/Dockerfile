From ubuntu:20.04

RUN apt-get update
RUN apt-get install -y software-properties-common
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
COPY vt_web_api.py .
COPY parse_result.py .
COPY tor.sh .
COPY requirements.txt .
COPY test.wasm /data/test.wasm
COPY entrypoint.sh .
RUN mkdir out
RUN mkdir upload

RUN pip3 install -r requirements.txt
RUN wget -O chromedriver.zip https://chromedriver.storage.googleapis.com/104.0.5112.79/chromedriver_linux64.zip
RUN unzip chromedriver.zip
RUN rm chromedriver.zip
# RUN mv chromedriver /bin/chromedriver
RUN touch name.socket

RUN ls .

ENV NO_WORKERS "8"
ENV WEB_USER "admin"
ENV WEB_PASS  "admin"


ENTRYPOINT ["/bin/bash", "entrypoint.sh"]
# Download the driver
