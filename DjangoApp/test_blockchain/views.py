from django.http import JsonResponse
from django.shortcuts import render
import requests
from django.http import HttpResponse
from django.views.decorators.csrf import csrf_exempt
import json
import subprocess
import os
from datetime import datetime
import qrcode
from io import BytesIO
from PIL import Image
import urllib.parse

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

def add_validators(request):
    return render(request, 'test_blockchain/add_validators.html')

def activate_ticket(request):
    return render(request, 'test_blockchain/activate_ticket.html')


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
            name = data.get("name")
            raw_date = data.get("date")

            if raw_date is not None:
                event_date = datetime.utcfromtimestamp(raw_date)
                human_readable_date = event_date.strftime("%Y-%m-%d")
            else:
                human_readable_date = "Unknown"

            # 1. Generujemy QR code z instrukcją / linkiem do dezaktywacji biletu
            qr_data = (
                f"eventId={urllib.parse.quote(str(event_id))}"
                f"&section={section.replace(' ', '!(_)!')}"
                f"&row={urllib.parse.quote(str(row))}"
                f"&seat={urllib.parse.quote(str(seat))}"
            )
            qr_img = qrcode.make(qr_data).convert("RGBA")

            base_dir = os.path.dirname(os.path.abspath(__file__))
            background_path = os.path.join(base_dir, "BiletInvilink.png")
            background = Image.open(background_path).convert("RGBA")

            # Ustalone wymiary przezroczystego obszaru (prostokąta) w tle:
            rect_min_x = 136
            rect_min_y = 24
            rect_width = 754 
            rect_height = 677

            # Skalujemy kod QR, by pasował do przezroczystego obszaru
            qr_resized = qr_img.resize((rect_width, rect_height))

            # Wklejamy kod QR do obrazu tła – jako, że QR ma przezroczyste tło, używamy go jako maski
            background.paste(qr_resized, (rect_min_x, rect_min_y), qr_resized)

            # Zapisujemy wynikowy obraz biletu do tymczasowego pliku
            ticket_filename = f"ticket_{event_id}_{row}_{seat}.png"
            background.save(ticket_filename)

            # Dodajemy plik biletu do IPFS
            result_ticket = subprocess.run(["ipfs", "add", ticket_filename], capture_output=True, text=True)
            if result_ticket.returncode != 0:
                os.remove(ticket_filename)
                return JsonResponse({"error": "Nie udało się dodać biletu do IPFS"}, status=500)

            parts_ticket = result_ticket.stdout.split()
            cid_ticket = parts_ticket[1] if len(parts_ticket) >= 2 else ""
            os.remove(ticket_filename)
            ticket_ipfs_link = f"ipfs://{cid_ticket}"

            # 2. Tworzymy metadane z obrazem ustawionym na wynikowy bilet (z wklejonym QR code)
            metadata = {
                "name": f"InviLink Ticket - {event_id}",
                "symbol": "INVI",
                "description": (
                    f"Ticket for {name}. This piece of art is a unique ticket for the event {event_id}. "
                    f"It is a proof of ownership of a seat in section {section}, row {row}, seat {seat}. "
                    "Before use activate it max 5 minutes before the event on the InviLink platform, "
                    "then show it at the entrance. After the event, you can keep it as a souvenir or sell "
                    "it to another person. Even used and after event it might have some value. "
                    "(even higher than the original price ;) ). And most important - HAVE A GREAT TIME AT THE EVENT!"
                ),
                "image": ticket_ipfs_link,
                "attributes": [
                    {"trait_type": "Section", "value": section},
                    {"trait_type": "Row", "value": row},
                    {"trait_type": "Seat", "value": seat},
                    {"trait_type": "Date of the event", "value": human_readable_date},
                ]
            }

            # Zapisujemy metadane do pliku tymczasowego
            metadata_filename = f"metadata_{event_id}_{row}_{seat}.json"
            with open(metadata_filename, "w") as f:
                json.dump(metadata, f)

            # Dodajemy plik metadanych do IPFS
            result_metadata = subprocess.run(["ipfs", "add", metadata_filename], capture_output=True, text=True)
            os.remove(metadata_filename)
            if result_metadata.returncode == 0:
                parts = result_metadata.stdout.split()
                cid_metadata = parts[1] if len(parts) >= 2 else ""
                uri = f"ipfs://{cid_metadata}"
                return JsonResponse({"uri": uri}, status=200)
            else:
                return JsonResponse({"error": "Nie udało się dodać pliku metadanych do IPFS"}, status=500)

        except Exception as e:
            return JsonResponse({"error": str(e)}, status=500)
    else:
        return JsonResponse({"error": "Tylko metoda POST jest obsługiwana"}, status=405)
    

@csrf_exempt
def deactivate_ticket(request):
    event_id = request.GET.get('eventId', 'Nie podano')
    section = request.GET.get('section', 'Nie podano')
    section = section.replace('!(_)!', ' ')
    row = request.GET.get('row', 'Nie podano')
    seat = request.GET.get('seat', 'Nie podano')

    context = {
        'event_id': event_id,
        'section': section,
        'row': row,
        'seat': seat,
    }
    return render(request, 'test_blockchain/deactivate_ticket.html', context)
