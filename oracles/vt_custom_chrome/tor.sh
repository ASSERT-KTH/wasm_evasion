tor &
TORID=$!
ELAPSED=0
while true
do
    sleep 5
    ((ELAPSED+=5))
    echo "Checking tor for restart"
    du -h ./local_tmp
    if [[ $(cat name.socket) == "RESTART" ]] 
    then
        echo "Restarting TOR"
        kill -9 $TORID
        tor &
        TORID=$!
        sleep 30
        # Wait for the circuit to create before setting the file
        echo "" > name.socket
        continue
    fi
    if [[ $ELAPSED -gt 600 ]] # Restart after 10 minutes
    then
        echo "Restarting TOR"
        kill -9 $TORID
        tor &
        TORID=$!
        sleep 30
        # Wait for the circuit to create before setting the file
        echo "" > name.socket
        ELAPSED=0
        continue
    fi
done

# watchdog