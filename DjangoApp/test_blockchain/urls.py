from . import views
from django.urls import path

urlpatterns = [
    path('phantom_test/', views.phantom_test, name='phantom_test'),
    path('nft_test/', views.nft_test, name='nft_test'),
]