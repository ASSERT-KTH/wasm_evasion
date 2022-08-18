
export PATH=$PATH:$(pwd)

/bin/bash tor.sh &
 
if [[ $1 == 'web' ]];
    then
        python3 vt_web_api.py 
    else 
        python3 vt_web_gui.py /data 
fi
