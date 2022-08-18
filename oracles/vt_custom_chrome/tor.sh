tor &
TORID=$!

while true
do
    sleep 5
    echo "Checking tor for restart"
    if [[ $(cat name.socket) == "RESTART" ]] 
    then
        echo "Restarting TOR"
        kill -9 $TORID
        tor &
        TORID=$!
        echo "" > name.socket
    fi
done

# watchdog