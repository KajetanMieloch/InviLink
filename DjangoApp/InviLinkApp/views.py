from django.shortcuts import render
from django.http import HttpResponseBadRequest
from django.http import HttpResponseServerError

def home(request):
    return render(request, "home.html")

def generate_400(request):
    return render(request, '400.html', status=400)

def generate_500(request):
    raise Exception("To jest błąd 500: Symulowany wyjątek")