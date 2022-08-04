sleep=$1
th=$2
shift 2
command="$@"


# launch process in background
$command 1> out.txt 2> logs.txt &
pid=$!

echo $pid
while true
do


    # sleep for a while and then check the mem
    sleep $sleep
    echo "Checking mem"
    ps -o pid,%mem,command -a | grep "analyzer"
    val=$(ps -o pid,%mem,command -a | grep analyzer | grep -v grep | awk {'print$2'})
    underlyingid=$(ps -o pid,%mem,command -a | grep analyzer | grep -v grep | awk {'print$1'})
    if [ -z "$val" ]; then
        echo "It seems the process is over"
        break
    fi
    
    echo "Mem usage " $val
    

    if (( $(echo "$val > $th" |bc -l) ))
    then
        # Restart and continue
        echo "Killing and restarting process"
        kill -9 $pid
        kill -9 $underlyingid
        # For testing
        $command 1>> out.txt 2>> logs.txt &
        pid=$!
    fi
done