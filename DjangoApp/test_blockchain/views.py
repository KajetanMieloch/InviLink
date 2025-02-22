from django.http import JsonResponse
from django.shortcuts import render
import requests
from django.http import HttpResponse
from selenium import webdriver
from selenium.webdriver.chrome.service import Service
from selenium.webdriver.common.by import By
from selenium.webdriver.chrome.options import Options


def phantom_test(request):
    return render(request, 'test_blockchain/phantom_test.html')

def nft_test(request):
    return render(request, 'test_blockchain/nft_test.html')

def nft_mint(request):
    return render(request, 'test_blockchain/nft_mint.html')

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