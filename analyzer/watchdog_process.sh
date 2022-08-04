sleep=$1
th=$2 # in Kb
shift 2
command="$@"


# launch process in background
$command  2> logs.txt &
pid=$!

echo $pid
while true
do


    # sleep for a while and then check the mem
    sleep $sleep
    echo "Checking mem"
    ps -o pid,rss,vsz,command ax | grep analyzer | grep -v grep | awk 'NR>1{$2=int($2/1024);$3=int($3/1024);}{print ;}'
    val=$(ps -o pid,rss,vsz,command ax | grep analyzer | grep -v grep | awk 'NR>1{$2=int($2/1024);$3=int($3/1024);}{print ;}' | awk {'print$2'})
    underlyingid=$(ps -o pid,command ax | grep analyzer | grep -v grep | awk {'print$1'})
    echo "Analyzer process id $underlyingid"

    if [ -z "$val" ]; then
        echo "It seems the process is over"
        break
    fi
    
    echo "Mem usage " $val "Mb"
    

    if (( $(echo "$val > $th" |bc -l) ))
    then
        # Restart and continue
        echo "Killing and restarting process"
        kill -9 $pid
        kill -9 $underlyingid
        # For testing
        $command 2>> logs.txt &
        pid=$!
    fi
done