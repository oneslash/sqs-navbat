#!/usr/bin/env bash
cd $(dirname "$0")
sqlite3 ../database.db < init.sql
