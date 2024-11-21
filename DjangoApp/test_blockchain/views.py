from django.shortcuts import render
from django.http import JsonResponse

def phantom_test(request):
    return render(request, 'test_blockchain/phantom_test.html')

def phantom_send(request):
    return JsonResponse({'error': 'Not implemented yet'}, status=501)

def phantom_send_mobile(request):
    return JsonResponse({'error': 'Not implemented yet'}, status=501)

def phantom_callback(request):
    return JsonResponse({'error': 'Not implemented yet'}, status=501)