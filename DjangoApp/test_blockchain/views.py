import base58
import base64
import urllib.parse
import json
import requests
import logging

from nacl.public import PrivateKey as NaClPrivateKey, PublicKey as NaClPublicKey, Box
from django.http import JsonResponse
from django.shortcuts import render, redirect
from solana.rpc.api import Client
from solana.transaction import Transaction
from solders.pubkey import Pubkey
from solders.signature import Signature
from base58 import b58decode

def phantom_test(request):
    return render(request, 'test_blockchain/phantom_test.html')


def get_blockchain_data(request, signature):
    network_url = "https://api.devnet.solana.com"
    client = Client(network_url)

    transaction_signature = signature

    # Fetch the transaction details using the updated method
    response = client.get_transaction(transaction_signature, encoding="jsonParsed")

    return JsonResponse(response)