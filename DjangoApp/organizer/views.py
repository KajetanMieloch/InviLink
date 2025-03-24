from django.shortcuts import render

# Create your views here.
def home(request):
    return render(request, 'organizer/home.html')

def events(request):
    return render(request, 'organizer/events.html')

def create(request):
    return render(request, 'organizer/create_event.html')

def explore(request):
    return render(request, 'organizer/explore_events.html')

def manage(request, event_id):
    return render(request, 'organizer/manage_event.html')