import json
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from django.contrib.auth import login
from django.contrib.auth.models import User
from userProfile.models import UserProfile


@csrf_exempt
def phantom_login(request):
    if request.method == "POST":
        # Obsługa logowania z API Phantom (desktop)
        try:
            data = json.loads(request.body)
            public_key = data.get("publicKey")
        except json.JSONDecodeError:
            return JsonResponse({"error": "Invalid JSON"}, status=400)

    elif request.method == "GET":
        # Obsługa logowania przez deep link (mobile)
        public_key = request.GET.get("public_key")

    else:
        return JsonResponse({"error": "Invalid request method"}, status=400)

    if public_key:
        # Znajdź lub utwórz użytkownika
        user, created = User.objects.get_or_create(username=public_key)
        login(request, user)

        # Upewnij się, że użytkownik ma profil i zaktualizuj klucz publiczny
        user_profile, _ = UserProfile.objects.get_or_create(user=user)
        user_profile.public_key = public_key
        user_profile.save()

        return JsonResponse({"status": "ok", "created": created})

    return JsonResponse({"error": "Missing public key"}, status=400)
