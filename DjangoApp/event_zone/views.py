from django.shortcuts import render
from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
import json
import subprocess
import os
from datetime import datetime
import qrcode
import urllib.parse
from PIL import Image
from django.http import JsonResponse
import traceback

def home(request):
    return render(request, "event_zone/home.html")

def events(request):
    return render(request, "event_zone/events.html")

def buy_ticket(request, event_id):
    return render(request, "event_zone/buy_ticket.html", {"event_id": event_id})

def tickets(request):
    return render(request, "event_zone/tickets.html")

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

            if None in [event_id, section, row, seat, name]:
                return JsonResponse({"error": "Brak wymaganych danych"}, status=400)

            if raw_date is not None:
                event_date = datetime.utcfromtimestamp(raw_date)
                human_readable_date = event_date.strftime("%Y-%m-%d")
            else:
                human_readable_date = "Unknown"

            # Generowanie kodu QR
            qr_data = (
                f"eventId={urllib.parse.quote(str(event_id))}"
                f"&section={section.replace(' ', '!(_)!')}"
                f"&row={urllib.parse.quote(str(row))}"
                f"&seat={urllib.parse.quote(str(seat))}"
            )
            qr_img = qrcode.make(qr_data).convert("RGBA")

            base_dir = os.path.dirname(os.path.abspath(__file__))
            background_path = os.path.join(base_dir, "BiletInvilink.png")
            if not os.path.exists(background_path):
                return JsonResponse({"error": "Plik tła nie został znaleziony"}, status=500)
            background = Image.open(background_path).convert("RGBA")

            rect_min_x = 136
            rect_min_y = 24
            rect_width = 754 
            rect_height = 677

            qr_resized = qr_img.resize((rect_width, rect_height))
            background.paste(qr_resized, (rect_min_x, rect_min_y), qr_resized)

            ticket_filename = f"ticket_{event_id}_{row}_{seat}.png"
            background.save(ticket_filename)

            # Dodanie biletu do IPFS
            result_ticket = subprocess.run(["ipfs", "add", ticket_filename], capture_output=True, text=True)
            if result_ticket.returncode != 0:
                error_msg = result_ticket.stderr.strip()
                os.remove(ticket_filename)
                return JsonResponse({"error": f"Nie udało się dodać biletu do IPFS: {error_msg}"}, status=500)

            parts_ticket = result_ticket.stdout.split()
            cid_ticket = parts_ticket[1] if len(parts_ticket) >= 2 else ""
            os.remove(ticket_filename)
            ticket_ipfs_link = f"ipfs://{cid_ticket}"

            # Generowanie metadanych
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

            metadata_filename = f"metadata_{event_id}_{row}_{seat}.json"
            with open(metadata_filename, "w") as f:
                json.dump(metadata, f)

            # Dodanie metadanych do IPFS
            result_metadata = subprocess.run(["ipfs", "add", metadata_filename], capture_output=True, text=True)
            os.remove(metadata_filename)
            if result_metadata.returncode != 0:
                error_msg = result_metadata.stderr.strip()
                return JsonResponse({"error": f"Nie udało się dodać metadanych do IPFS: {error_msg}"}, status=500)
            parts = result_metadata.stdout.split()
            cid_metadata = parts[1] if len(parts) >= 2 else ""
            uri = f"ipfs://{cid_metadata}"
            return JsonResponse({"uri": uri}, status=200)
        except Exception as e:
            traceback.print_exc()  # Wypisze pełny traceback w konsoli
            return JsonResponse({"error": str(e)}, status=500)
    return JsonResponse({"error": "Metoda niedozwolona"}, status=405)