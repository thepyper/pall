
START llama-server -hf unsloth/gemma-4-E4B-it-GGUF:Q6_K    -ngl 99 --port 4970 -c  65536 -fa auto
START llama-server -hf unsloth/Qwen3.6-35B-A3B-GGUF:Q4_K_M -ngl 0  --port 4971 -c 262144 -fa auto
pi --models fast,smart