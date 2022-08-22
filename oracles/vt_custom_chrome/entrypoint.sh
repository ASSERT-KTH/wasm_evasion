
export PATH=$PATH:$(pwd)

/bin/bash tor.sh &
 
sleep 30

if [[ $1 == 'web' ]];
    then
        python3 vt_web_api.py 
    else 
        echo "Standalone"
        python3 vt_web_gui.py /data 
fi
