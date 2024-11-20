from nacl.public import PrivateKey, PublicKey, Box
import base58
from django.http import JsonResponse

def phantom_callback(request):
    if request.method == "GET":
        phantom_encryption_public_key = request.GET.get('phantom_encryption_public_key')
        nonce_base58 = request.GET.get('nonce')
        encrypted_data_base58 = request.GET.get('data')

        if not all([phantom_encryption_public_key, nonce_base58, encrypted_data_base58]):
            return JsonResponse({'error': 'Missing parameters'}, status=400)

        try:
            # Retrieve and decode the private key from session
            private_key_base58 = request.session.get('dapp_private_key')
            if not private_key_base58:
                return JsonResponse({'error': 'Private key not found in session'}, status=400)
            private_key_bytes = base58.b58decode(private_key_base58)
            private_key = PrivateKey(private_key_bytes)

            # Decode Phantom's public key
            phantom_public_key_bytes = base58.b58decode(phantom_encryption_public_key)
            phantom_public_key = PublicKey(phantom_public_key_bytes)

            # Create Box for encryption/decryption
            box = Box(private_key, phantom_public_key)
            nonce = base58.b58decode(nonce_base58)
            encrypted_data = base58.b58decode(encrypted_data_base58)

            decrypted_message = box.decrypt(encrypted_data, nonce).decode('utf-8')

            return JsonResponse({'status': 'success', 'message': decrypted_message})

        except Exception as e:
        # Detailed logging for debugging
            print("Exception during decryption:", str(e))
            return JsonResponse({'error': 'Decryption failed', 'details': str(e)}, status=400)
    else:
        return JsonResponse({'error': 'Invalid request method'}, status=405)