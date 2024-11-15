from django.shortcuts import render
from django.contrib.auth.decorators import login_required
from solana.rpc.api import Client
from solana.publickey import PublicKey
from solana.transaction import Transaction, TransactionInstruction
from solders.keypair import Keypair  # Użyj, aby stworzyć podpisującego

@login_required
def initialize_blockchain(request):
    # Konfiguracja klienta Solany
    client = Client("https://api.devnet.solana.com")

    # Adres programu Solana (to Twój program z ID na Devnet)
    program_id = PublicKey("EEnUgo8XJYHQkQ14biGpu21KCtESb3h5AJ4iFFJsQHVA")

    # Klucz podpisującego (testowy klucz podpisujący, nie używaj go w produkcji)
    signer = Keypair()

    # Klucz publiczny użytkownika (testowy klucz publiczny)
    user_public_key = signer.pubkey()

    # Utwórz transakcję
    tx = Transaction()

    # Utwórz instrukcję
    instruction = TransactionInstruction(
        keys=[
            {"pubkey": user_public_key, "is_signer": True, "is_writable": False}
        ],
        program_id=program_id,
        data=b''  # Dane mogą być puste, ponieważ chcemy jedynie wywołać 'initialize'
    )

    # Dodaj instrukcję do transakcji
    tx.add(instruction)

    try:
        # Wyślij transakcję
        response = client.send_transaction(tx, signer)
        return render(request, 'initialize_blockchain.html', {'message': f"Transaction successful: {response}"})
    except Exception as e:
        return render(request, 'initialize_blockchain.html', {'message': f"Transaction failed: {str(e)}"})
