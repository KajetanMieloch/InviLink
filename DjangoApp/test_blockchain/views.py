from django.contrib.auth.decorators import login_required
from django.http import JsonResponse
from solana.rpc.api import Client
from userProfile.models import UserProfile
from django.shortcuts import render, redirect
import base58
from nacl.public import PrivateKey as NaClPrivateKey, PublicKey as NaClPublicKey, Box
from solana.publickey import PublicKey as SolanaPublicKey
from solana.system_program import TransferParams, transfer
from solana.transaction import Transaction
import base64
import urllib.parse
import json
import requests

solana_client = Client("https://api.devnet.solana.com")

def phantom_test(request):
    return render(request, 'test_blockchain/phantom_test.html')

def decrypt_data(request, phantom_encryption_public_key, nonce_base58, encrypted_data_base58):
    private_key_base58 = request.session.get('dapp_private_key')
    if not private_key_base58:
        raise Exception('Private key not found in session')
    private_key_bytes = base58.b58decode(private_key_base58)
    private_key = NaClPrivateKey(private_key_bytes)

    phantom_public_key_bytes = base58.b58decode(phantom_encryption_public_key)
    phantom_public_key = NaClPublicKey(phantom_public_key_bytes)

    box = Box(private_key, phantom_public_key)
    nonce = base58.b58decode(nonce_base58)
    encrypted_data = base58.b58decode(encrypted_data_base58)

    decrypted_message = box.decrypt(encrypted_data, nonce).decode('utf-8')
    return decrypted_message

def get_recent_blockhash():
    # Solana Devnet RPC URL
    devnet_url = "https://api.devnet.solana.com"

    # Payload for getLatestBlockhash
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getLatestBlockhash",
        "params": [
            {
                "commitment": "finalized"
            }
        ]
    }

    # Make the request to the devnet
    response = requests.post(devnet_url, json=payload)

    # Parse and print the response
    if response.status_code == 200:
        blockhash_data = response.json()
        print("Latest Blockhash:", blockhash_data['result']['value']['blockhash'])
        return blockhash_data  # Return the blockhash data
    else:
        print("Error:", response.text)
        return {'error': response.text} 

def phantom_callback(request):
    if request.method == "GET":
        phantom_encryption_public_key = request.GET.get('phantom_encryption_public_key')
        nonce_base58 = request.GET.get('nonce')
        encrypted_data_base58 = request.GET.get('data')

        if not all([phantom_encryption_public_key, nonce_base58, encrypted_data_base58]):
            return JsonResponse({'error': 'Missing parameters'}, status=400)
        
        try:
            decrypted_message = decrypt_data(request, phantom_encryption_public_key, nonce_base58, encrypted_data_base58)
            decrypted_data = json.loads(decrypted_message)
            user_public_key_str = decrypted_data.get('public_key')
            if not user_public_key_str:
                return JsonResponse({'error': 'Public key not found in decrypted data'}, status=400)
            user_public_key = SolanaPublicKey(user_public_key_str)
        except Exception as e:
            print("Exception during decryption:", str(e))
            return JsonResponse({'error': 'Decryption failed', 'details': str(e)}, status=400)
        
        # Get the latest blockhash
        try:
            print("Fetching latest blockhash...")
            blockhash_response = get_recent_blockhash()
            print("blockhash_response:", blockhash_response)

            if 'error' in blockhash_response:
                print("Error fetching latest blockhash:", blockhash_response['error'])
                return JsonResponse({'error': 'Failed to get latest blockhash', 'details': blockhash_response['error']}, status=500)
            elif 'result' not in blockhash_response:
                print("Unexpected response structure:", blockhash_response)
                return JsonResponse({'error': 'Unexpected response structure when fetching blockhash'}, status=500)

            recent_blockhash = blockhash_response['result']['value']['blockhash']
        except Exception as e:
            print("Exception when fetching blockhash:", str(e))
            return JsonResponse({'error': 'Exception when fetching blockhash', 'details': str(e)}, status=500)

        # Create the transfer instruction
        recipient_public_key = SolanaPublicKey('4XC8cuNEpwRiASbD817cX6dcH8EsNfcBUjAv9sDi9mgp')
        amount_lamports = 1000000000  # 1 SOL

        transfer_instruction = transfer(
            TransferParams(
                from_pubkey=user_public_key,
                to_pubkey=recipient_public_key,
                lamports=amount_lamports
            )
        )

        # Create the transaction
        transaction = Transaction()
        transaction.add(transfer_instruction)
        transaction.recent_blockhash = recent_blockhash
        transaction.fee_payer = user_public_key

        # Serialize and encode the transaction in Base64
        serialized_tx = transaction.serialize_message()
        tx_base64 = base64.b64encode(serialized_tx).decode('utf-8')

        # Build the URL for `signAndSendTransaction`
        params = {
            'tx': tx_base64,
            'redirect_link': 'http://192.168.1.211:8000/test_blockchain/phantom_success',
            'app_url': 'http://192.168.1.211:8000',
            'app_title': 'InviLink',
            'cluster': 'devnet'
        }
        encoded_params = {k: urllib.parse.quote(v) for k, v in params.items()}
        base_url = 'https://phantom.app/ul/v1/signAndSendTransaction'
        query_string = '&'.join(f'{k}={v}' for k, v in encoded_params.items())
        full_url = f'{base_url}?{query_string}'

        # Redirect the user to Phantom
        return redirect(full_url)

def phantom_success(request):
    if request.method == "GET":
        request_data = request.GET
        return JsonResponse(request_data)
