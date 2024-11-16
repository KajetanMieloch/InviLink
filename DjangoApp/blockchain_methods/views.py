from django.shortcuts import render
from django.contrib.auth.decorators import login_required
from solana.rpc.api import Client


http_client = Client("https://api.devnet.solana.com")

@login_required
def initialize_blockchain(request):
    pass