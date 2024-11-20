from nacl.public import PrivateKey, PublicKey, Box
import base58
from django.http import JsonResponse
from django.shortcuts import redirect
from django.contrib.auth import login
from django.contrib.auth.models import User
from userProfile.models import UserProfile
from django.views.decorators.csrf import csrf_exempt
import json
from django.urls import reverse

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

            return redirect(reverse('phantom_mobile_login') + f'?public_key={decrypted_message}')

        except Exception as e:
            print("Exception during decryption:", str(e))
            return JsonResponse({'error': 'Decryption failed', 'details': str(e)}, status=400)
    else:
        return JsonResponse({'error': 'Invalid request method'}, status=405)


@csrf_exempt
def phantom_mobile_login(request):
    if request.method == "GET":
        try:
            json_data = json.loads(request.GET.get('public_key'))
            public_key = json_data.get("public_key")
        except (TypeError, json.JSONDecodeError):
            return JsonResponse({"error": "Invalid public key format"}, status=400)

        if public_key:
            # Find or create the user
            user, created = User.objects.get_or_create(username=public_key)
            login(request, user)

            # Ensure the user has a profile and update the public key
            user_profile, _ = UserProfile.objects.get_or_create(user=user)
            user_profile.public_key = public_key
            user_profile.save()

            # Redirect to the events page
            return redirect('/')

        return JsonResponse({"error": "Missing public key"}, status=400)
    else:
        return JsonResponse({"error": "Invalid request method"}, status=405)