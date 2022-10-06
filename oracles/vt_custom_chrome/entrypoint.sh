
export PATH=$PATH:$(pwd)
mc config host add $MC_ENDPOINT $MINIO_HOST $MINIO_USER $MINIO_PASS

/bin/bash tor.sh &
 
sleep 30

if [[ $1 == 'web' ]];
    then
        python3 vt_web_api.py 
    else 
        if [[ $1 == 'worker' ]]
        then
            echo "Worker"
            # Launch workers
        else
            echo "Standalone"
            python3 vt_web_gui.py /data 
        fi
fi
