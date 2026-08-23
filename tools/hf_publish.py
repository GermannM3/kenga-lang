#!/usr/bin/env python3
"""Publish Kenga-Prophet weights to Hugging Face.

Token is read from the environment (HF_TOKEN or HUGGINGFACE_TOKEN).
Never hardcode it: this repo is public and GitHub Push Protection
blocks pushes containing secrets.
"""
import os
import sys
from huggingface_hub import HfApi, create_repo, upload_folder

TOKEN = os.environ.get("HF_TOKEN") or os.environ.get("HUGGINGFACE_TOKEN")
if not TOKEN:
    sys.exit("set HF_TOKEN (or HUGGINGFACE_TOKEN) in the environment")

REPO_ID = os.environ.get("KENGA_HF_REPO", "GermannM/kenga-prophet-m3")
FOLDER = "hf/Kenga-prophet-m3"

api = HfApi(token=TOKEN)

print(f"creating repo {REPO_ID}...")
create_repo(REPO_ID, token=TOKEN, repo_type="model", exist_ok=True)

print(f"uploading {FOLDER}/...")
result = upload_folder(
    folder_path=FOLDER,
    repo_id=REPO_ID,
    repo_type="model",
    token=TOKEN,
    commit_message="Kenga Prophet M3: numpy transformer decoder (~11.1K params) on Kenga corpus, 71.9% held-out token accuracy",
)
print("uploaded:")
print(result)
