#!/bin/bash
set -e

echo "Starting Bitcoin Core..."
$BITCOIND_EXE -daemon

echo "Waiting for Bitcoin Core to start..."
sleep 10

echo "Starting Electrs..."
$ELECTRS_EXE -vvv --network regtest --daemon-dir /home/appuser/.bitcoin --db-dir /home/appuser/db

echo "Starting main application..."
/bin/server