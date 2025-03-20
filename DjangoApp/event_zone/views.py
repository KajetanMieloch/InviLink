from django.shortcuts import render

def home(request):
    return render(request, "event_zone/home.html")
