from django.urls import path
from . import views

urlpatterns = [
    path('initialize/', views.initialize_blockchain, name='initialize_blockchain'),
]
