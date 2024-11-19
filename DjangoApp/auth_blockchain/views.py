from django.shortcuts import render
from django.http import JsonResponse

def phantom_callback(request):
    public_key = request.GET.get('public_key')
    if public_key:
        request.session['phantom_public_key'] = public_key
        return JsonResponse({'status': 'success', 'public_key': public_key})
    return JsonResponse({'status': 'error', 'message': 'Public key not found'})
