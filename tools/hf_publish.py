#!/usr/bin/env python3
"""Publish Kenga-Prophet weights to Hugging Face."""
import os
import sys
from huggingface_hub import HfApi, create_repo, upload_folder

TOKEN = "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"
REPO_ID = os.environ.get("KENGA_HF_REPO", "GermannM/kenga-prophet-m2-k16")
FOLDER = "hf/Kenga-prophet-m2-k16"

api = HfApi(token=TOKEN)

print(f"creating repo {REPO_ID}...")
create_repo(REPO_ID, token=TOKEN, repo_type="model", exist_ok=True)

print(f"uploading {FOLDER}/...")
result = upload_folder(
    folder_path=FOLDER,
    repo_id=REPO_ID,
    repo_type="model",
    token=TOKEN,
    commit_message="Kenga Prophet M2: linear token predictor (~6.3K params) on Kenga corpus, 21.4% held-out token accuracy",
)
print("uploaded:")
print(result)
