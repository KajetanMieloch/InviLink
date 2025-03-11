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
            name = data.get("name")
            raw_date = data.get("date")

            # Konwersja timestampa na datę
            if raw_date is not None:
                event_date = datetime.utcfromtimestamp(raw_date)
                human_readable_date = event_date.strftime("%Y-%m-%d %H:%M:%S")
            else:
                human_readable_date = "Unknown"

            # 1. Generujemy QR code z instrukcją / linkiem
            qr_url = (
                f"https://invilink.bieda.it/test_blockchain/deactivate_ticket"
                f"?eventId={event_id}&section={section}&row={row}&seat={seat}"
            )

            # Tworzymy obrazek w pamięci
            qr_img = qrcode.make(qr_url)
            qr_filename = f"qr_{event_id}_{section}_{row}_{seat}.png"
            qr_img.save(qr_filename)

            # Dodajemy plik QR do IPFS
            result_qr = subprocess.run(["ipfs", "add", qr_filename], capture_output=True, text=True)
            if result_qr.returncode != 0:
                # Błąd w dodawaniu do IPFS
                os.remove(qr_filename)
                return JsonResponse({"error": "Nie udało się dodać QR do IPFS"}, status=500)

            # Wyciągamy CID z wyniku "ipfs add"
            parts_qr = result_qr.stdout.split()
            cid_qr = parts_qr[1] if len(parts_qr) >= 2 else ""
            # Można od razu usunąć plik lokalny QR
            os.remove(qr_filename)

            # Link do obrazka w IPFS
            qr_ipfs_link = f"ipfs://{cid_qr}"

            # 2. Tworzymy metadane
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
                # Zamiast losowego obrazka – link do wygenerowanego kodu QR w IPFS
                "image": qr_ipfs_link,
                "attributes": [
                    {"trait_type": "Section", "value": section},
                    {"trait_type": "Row", "value": row},
                    {"trait_type": "Seat", "value": seat},
                    {"trait_type": "Date of the event", "value": human_readable_date},
                ]
            }

            # Zapisujemy metadane do pliku
            metadata_filename = f"metadata_{event_id}_{row}_{seat}.json"
            with open(metadata_filename, "w") as f:
                json.dump(metadata, f)

            # 3. Dodajemy plik metadanych do IPFS
            result_metadata = subprocess.run(["ipfs", "add", metadata_filename], capture_output=True, text=True)
            # Usuwamy plik z dysku
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