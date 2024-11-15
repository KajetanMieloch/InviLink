import json
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from django.contrib.auth import login
from django.contrib.auth.models import User


@csrf_exempt
def phantom_login(request):
    if request.method == "POST":
        data = json.loads(request.body)
        public_key = data.get("publicKey")

        # Szukaj użytkownika po publicznym kluczu (adresie Solana) lub utwórz nowego
        user, created = User.objects.get_or_create(username=public_key)
        login(request, user)

        return JsonResponse({"status": "ok"})
    return JsonResponse({"error": "Invalid request"}, status=400)
