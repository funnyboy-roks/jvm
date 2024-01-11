build_all () {
    echo "Rebuilding all files in directory"
    for f in *.java; do
        echo "    Compiling $f"
        javac $f
    done
}

if [[ $# -eq 0 ]]; then 
    set -xe
fi
build_all

if [[ $1 = "watch" ]]; then
    echo "Watching..."
    while inotifywait -e close_write .; do
        build_all
    done
fi
