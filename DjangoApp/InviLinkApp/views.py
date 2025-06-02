from django.shortcuts import render
from django.http import HttpResponseBadRequest
from django.http import HttpResponseServerError

def home(request):
    return render(request, "home.html")

def generate_400(request):
    return HttpResponseBadRequest("To jest błąd 400: Niepoprawne żądanie")

def generate_500(request):
    raise Exception("To jest błąd 500: Symulowany wyjątek")