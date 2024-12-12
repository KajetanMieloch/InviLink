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

