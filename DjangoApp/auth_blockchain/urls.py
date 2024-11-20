from django.urls import path
from . import views

urlpatterns = [
    path('auth/phantom/callback', views.phantom_callback, name='phantom_callback'),
]
