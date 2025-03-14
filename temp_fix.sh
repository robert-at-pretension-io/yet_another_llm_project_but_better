#!/bin/bash
sed -i '208s/.*headers modifier should be.*/                  "The headers modifier should match Bearer token for API key");/' tests/executable_blocks_test.rs
