#!/bin/bash
# Shell test file

main() {
    echo "Hello Shell"
}

add_numbers() {
    local a=$1
    local b=$2
    echo $((a + b))
}

fibonacci() {
    local n=$1
    if [ $n -le 1 ]; then
        echo $n
    else
        local prev1=$(fibonacci $((n - 1)))
        local prev2=$(fibonacci $((n - 2)))
        echo $((prev1 + prev2))
    fi
}

is_prime() {
    local n=$1
    if [ $n -le 1 ]; then
        return 1
    fi

    for ((i=2; i*i<=n; i++)); do
        if [ $((n % i)) -eq 0 ]; then
            return 1
        fi
    done
    return 0
}

# Call main function
main