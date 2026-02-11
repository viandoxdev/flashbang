#!/bin/bash

openssl base64 < "./debug.keystore" | tr -d '\n' > "./debug.keystore.encoded"
