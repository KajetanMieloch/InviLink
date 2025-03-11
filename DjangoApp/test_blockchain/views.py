from django.http import JsonResponse
from django.shortcuts import render
import requests
from django.http import HttpResponse
from django.views.decorators.csrf import csrf_exempt
import json
import subprocess
import os


def init_org_pool(request):
    return render(request, 'test_blockchain/init_org_pool.html')

def add_org(request):
    return render(request, 'test_blockchain/add_org.html')

def remove_org(request):
    return render(request, 'test_blockchain/remove_org.html')

def create_event(request):
    return render(request, 'test_blockchain/create_event.html')

def init_event_reg(request):
    return render(request, 'test_blockchain/init_event_reg.html')

def close_account(request):
    return render(request, 'test_blockchain/close_account.html')

def explore_events(request):
    return render(request, 'test_blockchain/explore_events.html')

def manage_events(request):
    return render(request, 'test_blockchain/manage_events.html')

def init_dictionary(request):
    return render(request, 'test_blockchain/init_dictionary.html')

def init(request):
    return render(request, 'test_blockchain/init.html')

def user_event(request):
    return render(request, 'test_blockchain/user_event.html')

def event_detail(request, event_id):
    return render(request, 'test_blockchain/event_detail.html', {'event_id': event_id})

def test_mint(request):
    return render(request, 'test_blockchain/test_mint.html')


@csrf_exempt
def generate_metadata(request):
    print("generate_metadata")
    if request.method == "POST":
        try:
            data = json.loads(request.body)
            event_id = data.get("eventId")
            section = data.get("section")
            row = data.get("row")
            seat = data.get("seat")
            date = data.get("date")
            name = data.get("name")

            metadata = {
                "name": f"InviLink Ticket - {event_id}",
                "symbol": "INVI",
                "description": f"Ticket for {name}. This piece of art is a unique ticket for the event {event_id}. It is a proof of ownership of a seat in section {section}, row {row}, seat {seat}. Before use activate it max 5 minutes before the event on the InviLink platform, then show it at the entrance. After the event, you can keep it as a souvenir or sell it to another person. Even used and after event it might have some value. (even higher than the original price ;) ). And most important - HAVE A GREAT TIME AT THE EVENT!",
                "image": "https://upload.wikimedia.org/wikipedia/commons/8/88/Royle%27sPika-Ochotona_roylei-Tungnath-Uttarkhand-India-3thJune2013.jpg",
                "attributes": [
                    {"trait_type": "Section", "value": section},
                    {"trait_type": "Row", "value": row},
                    {"trait_type": "Seat", "value": seat},
                    {"trait_type": "Date of the event", "value": date},
                ]
            }

            # Tworzymy unikalny plik metadata.json
            filename = f"metadata_{event_id}_{row}_{seat}.json"
            with open(filename, "w") as f:
                json.dump(metadata, f)

            # Dodajemy plik do IPFS
            result = subprocess.run(["ipfs", "add", filename], capture_output=True, text=True)
            os.remove(filename)  # Usuwamy lokalny plik po wrzuceniu do IPFS

            if result.returncode == 0:
                # Oczekujemy wyniku w formacie: "added <CID> filename"
                parts = result.stdout.split()
                cid = parts[1] if len(parts) >= 2 else ""
                uri = f"ipfs://{cid}"
                return JsonResponse({"uri": uri}, status=200)
            else:
                return JsonResponse({"error": "Nie udało się dodać pliku do IPFS"}, status=500)
        except Exception as e:
            return JsonResponse({"error": str(e)}, status=500)
    else:
        return JsonResponse({"error": "Tylko metoda POST jest obsługiwana"}, status=405)