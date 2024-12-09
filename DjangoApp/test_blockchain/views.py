import base58
import base64
import urllib.parse
import json
import requests
import logging

from nacl.public import PrivateKey as NaClPrivateKey, PublicKey as NaClPublicKey, Box
from django.http import JsonResponse
from django.shortcuts import render, redirect
from solana.publickey import PublicKey as SolanaPublicKey
from solana.rpc.api import Client
from solana.system_program import TransferParams, transfer
from solana.transaction import Transaction

# Configure logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)

# Initialize Solana client for Devnet
solana_client = Client("https://api.devnet.solana.com")

def phantom_test(request):
    return render(request, 'test_blockchain/phantom_test.html')


from django.http import JsonResponse
from solana.rpc.api import Client
from solana.publickey import PublicKey
import base64

def get_blockchain_data(request):
    # Ustawienia połączenia z blockchainem
    network_url = "https://api.devnet.solana.com"
    client = Client(network_url)

    # Publiczny klucz konta, z którego odczytujemy dane
    account_public_key = PublicKey("5iHT8dpa6TJssXi3VXGAa1W7nVzGxT18dj6PocxW81m9")

    try:
        # Pobranie danych z konta
        response = client.get_account_info(account_public_key)

        if response["result"]["value"] is None:
            return JsonResponse({"error": "Account not found"}, status=404)

        # Dane w formacie Base64 (musimy je zdekodować)
        raw_data = response["result"]["value"]["data"][0]
        decoded_data = base64.b64decode(raw_data).decode("utf-8")  # Przykład: dane jako tekst

        return JsonResponse({"blockchain_data": decoded_data})

    except Exception as e:
        return JsonResponse({"error": str(e)}, status=500)



# def decrypt_data(request, phantom_encryption_public_key, nonce_base58, encrypted_data_base58):
#     logger.debug("Starting decryption process.")
#     private_key_base58 = request.session.get('dapp_private_key')
#     if not private_key_base58:
#         error_msg = 'Private key not found in session'
#         logger.error(error_msg)
#         raise Exception(error_msg)

#     private_key_bytes = base58.b58decode(private_key_base58)
#     private_key = NaClPrivateKey(private_key_bytes)
#     logger.debug("Private key obtained from session.")

#     phantom_public_key_bytes = base58.b58decode(phantom_encryption_public_key)
#     phantom_public_key = NaClPublicKey(phantom_public_key_bytes)
#     logger.debug("Phantom's public key decoded.")

#     box = Box(private_key, phantom_public_key)
#     nonce = base58.b58decode(nonce_base58)
#     encrypted_data = base58.b58decode(encrypted_data_base58)
#     logger.debug("Nonce and encrypted data decoded.")

#     decrypted_message = box.decrypt(encrypted_data, nonce).decode('utf-8')
#     logger.debug("Decryption successful. Decrypted message: %s", decrypted_message)
#     return decrypted_message

# def get_recent_blockhash():
#     logger.debug("Fetching recent blockhash from Solana Devnet.")
#     devnet_url = "https://api.devnet.solana.com"

#     payload = {
#         "jsonrpc": "2.0",
#         "id": 1,
#         "method": "getLatestBlockhash",
#         "params": [
#             {
#                 "commitment": "finalized"
#             }
#         ]
#     }

#     response = requests.post(devnet_url, json=payload)

#     if response.status_code == 200:
#         blockhash_data = response.json()
#         logger.debug("Blockhash data received: %s", blockhash_data)
#         return blockhash_data  # Return the blockhash data
#     else:
#         error_msg = f"Error fetching blockhash: {response.text}"
#         logger.error(error_msg)
#         return {'error': error_msg}

# def phantom_callback(request):
#     if request.method == "GET":
#         logger.debug("Received GET request for phantom_callback.")
#         phantom_encryption_public_key = request.GET.get('phantom_encryption_public_key')
#         nonce_base58 = request.GET.get('nonce')
#         encrypted_data_base58 = request.GET.get('data')

#         if not all([phantom_encryption_public_key, nonce_base58, encrypted_data_base58]):
#             error_msg = 'Missing parameters in the request.'
#             logger.error(error_msg)
#             return JsonResponse({'error': error_msg}, status=400)

#         try:
#             decrypted_message = decrypt_data(request, phantom_encryption_public_key, nonce_base58, encrypted_data_base58)
#             decrypted_data = json.loads(decrypted_message)
#             user_public_key_str = decrypted_data.get('public_key')
#             if not user_public_key_str:
#                 error_msg = 'Public key not found in decrypted data.'
#                 logger.error(error_msg)
#                 return JsonResponse({'error': error_msg}, status=400)
#             user_public_key = SolanaPublicKey(user_public_key_str)
#             logger.debug("User's public key obtained: %s", user_public_key)
#         except Exception as e:
#             logger.exception("Exception during decryption:")
#             return JsonResponse({'error': 'Decryption failed', 'details': str(e)}, status=400)

#         # Get the latest blockhash
#         try:
#             blockhash_response = get_recent_blockhash()
#             logger.debug("Blockhash response: %s", blockhash_response)

#             if 'error' in blockhash_response:
#                 logger.error("Error fetching latest blockhash: %s", blockhash_response['error'])
#                 return JsonResponse({'error': 'Failed to get latest blockhash', 'details': blockhash_response['error']}, status=500)
#             elif 'result' not in blockhash_response:
#                 logger.error("Unexpected response structure when fetching blockhash: %s", blockhash_response)
#                 return JsonResponse({'error': 'Unexpected response structure when fetching blockhash'}, status=500)

#             recent_blockhash = blockhash_response['result']['value']['blockhash']
#             logger.debug("Recent blockhash obtained: %s", recent_blockhash)
#         except Exception as e:
#             logger.exception("Exception when fetching blockhash:")
#             return JsonResponse({'error': 'Exception when fetching blockhash', 'details': str(e)}, status=500)

#         try:
#             # Pobierz saldo użytkownika
#             balance = solana_client.get_balance(user_public_key)
#             logger.debug("User's balance: %d lamports", balance['result']['value'])
#             if balance['result']['value'] < 1000:
#                 logger.error("Insufficient balance for transaction.")
#                 return JsonResponse({'error': 'Insufficient balance for transaction'}, status=400)
#         except Exception as e:
#             logger.exception("Exception during balance check:")
#             return JsonResponse({'error': 'Balance check failed', 'details': str(e)}, status=500)


#         # Create the transfer instruction
#         try:
#             recipient_public_key = SolanaPublicKey('4Wg5ZqjS3AktHzq34hK1T55aFNKSjBpmJ3PyRChpPNDh')
#             amount_lamports = 1000  # 0.000001 SOL
#             logger.debug("Recipient's public key: %s", recipient_public_key)
#             logger.debug("Amount to transfer (lamports): %d", amount_lamports)

#             transfer_instruction = transfer(
#                 TransferParams(
#                     from_pubkey=user_public_key,
#                     to_pubkey=recipient_public_key,
#                     lamports=amount_lamports
#                 )
#             )
#             logger.debug("Transfer instruction created.")
#         except Exception as e:
#             logger.exception("Exception during transfer instruction creation:")
#             return JsonResponse({'error': 'Transfer instruction creation failed', 'details': str(e)}, status=500)

#         # Create the transaction
#         try:
#             transaction = Transaction()
#             transaction.add(transfer_instruction)
#             transaction.recent_blockhash = recent_blockhash
#             transaction.fee_payer = user_public_key
#             logger.debug("Transaction created with fee payer set.")

#             # Serialize and encode the transaction in Base64
#             serialized_tx = transaction.serialize_message()
#             logger.debug("Serialized Transaction (bytes): %s", serialized_tx)
#             logger.debug("Serialized Transaction (hex): %s", serialized_tx.hex())

#             tx_base64 = base64.b64encode(serialized_tx).decode('utf-8')
#             logger.debug("Base64 Encoded Transaction: %s", tx_base64)
#         except Exception as e:
#             logger.exception("Exception during transaction construction or serialization:")
#             return JsonResponse({'error': 'Transaction construction failed', 'details': str(e)}, status=500)

#         # Build the URL for `signAndSendTransaction`
#         try:
#             params = {
#                 'tx': tx_base64,
#                 'redirect_link': 'http://192.168.1.211:8000/test_blockchain/phantom_success',
#                 'app_url': 'http://192.168.1.211:8000',
#                 'app_title': 'InviLink',
#                 'cluster': 'devnet'
#             }
#             logger.debug("Parameters before encoding: %s", params)

#             # URL-encode the parameters
#             encoded_params = {k: urllib.parse.quote(str(v), safe='') for k, v in params.items()}
#             logger.debug("Encoded Parameters: %s", encoded_params)

#             base_url = 'https://phantom.app/ul/v1/signAndSendTransaction'
#             query_string = '&'.join(f'{k}={v}' for k, v in encoded_params.items())
#             full_url = f'{base_url}?{query_string}'
#             logger.debug("Full URL to redirect to: %s", full_url)
#         except Exception as e:
#             logger.exception("Exception during URL construction:")
#             return JsonResponse({'error': 'URL construction failed', 'details': str(e)}, status=500)

#         # Redirect the user to Phantom
#         return redirect(full_url)
#     else:
#         logger.error("Invalid request method: %s", request.method)
#         return JsonResponse({'error': 'Invalid request method'}, status=405)

# def phantom_success(request):
#     if request.method == "GET":
#         request_data = request.GET
#         logger.debug("Phantom success callback received data: %s", request_data)
#         return JsonResponse(request_data)
#     else:
#         logger.error("Invalid request method: %s", request.method)
#         return JsonResponse({'error': 'Invalid request method'}, status=405)
