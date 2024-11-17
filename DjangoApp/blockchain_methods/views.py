import json
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from django.contrib.auth.decorators import login_required
from django.shortcuts import render
from solana.rpc.api import Client
from solana.transaction import Transaction
from solana.system_program import TransferParams, transfer
from solana.publickey import PublicKey
from django.conf import settings

http_client = Client("https://api.devnet.solana.com")

@csrf_exempt
@login_required
def initialize_blockchain(request):
    return render(request, 'initialize_blockchain.html')

@csrf_exempt
@login_required
def send_sol(request):
    pass
