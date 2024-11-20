from nacl.public import PrivateKey
import base58

def encryption_keys(request):
    # Generuj klucze jednorazowe tylko raz na sesję
    if 'dapp_private_key' not in request.session:
        private_key = PrivateKey.generate()
        public_key = private_key.public_key

        # Kodowanie kluczy w Base58
        private_key_base58 = base58.b58encode(private_key.encode()).decode('utf-8')
        public_key_base58 = base58.b58encode(public_key.encode()).decode('utf-8')

        # Przechowywanie klucza prywatnego w sesji
        request.session['dapp_private_key'] = private_key_base58
        request.session['dapp_public_key'] = public_key_base58
    else:
        public_key_base58 = request.session['dapp_public_key']

    # Zwrócenie klucza publicznego do szablonu
    return {
        'dapp_encryption_public_key': public_key_base58,
    }
