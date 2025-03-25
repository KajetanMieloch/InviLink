from django.shortcuts import render

def home(request):
    return render(request, "event_zone/home.html")

def events(request):
    return render(request, "event_zone/events.html")

def buy_ticket(request, event_id):
    return render(request, "event_zone/buy_ticket.html", {"event_id": event_id})