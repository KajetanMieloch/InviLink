from django.urls import path
from . import views

urlpatterns = [
    path('initialize/', views.initialize_blockchain, name='initialize_blockchain'),
    path('send_sol/', views.send_sol, name='send_sol'),
]
